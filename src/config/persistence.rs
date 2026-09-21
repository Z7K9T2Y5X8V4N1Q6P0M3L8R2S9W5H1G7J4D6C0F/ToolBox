//! Config file load and save logic.
//!
//! The config file is stored at `{user config dir}/{app name}/CONFIG.toml`.
//! Under normal circumstances, this resolves to `%APPDATA%\{app name}\CONFIG.toml`.
//!
//! When running under high-privilege system tokens (such as `TrustedInstaller`),
//! standard environment variables resolve to `systemprofile`. To ensure user preferences
//! persist correctly for the interactive user—across both local console sessions and
//! remote desktop (RDP) multi-user environments—this module resolves the session token
//! of the active desktop session and queries the true roaming AppData path.
//!
//! # Load behavior
//! - If the file does not exist, a default config is written and returned.
//! - If the file exists but cannot be parsed, an error dialog is shown,
//!   the default config is written over the corrupt file, and returned.
//! - If the file exists and parses successfully, it is returned as-is.

use std::{ffi::OsString, fs, os::windows::ffi::OsStringExt, path::PathBuf};

use anyhow::{Context, Result};
use rust_i18n::t;
use serde::{Deserialize, Serialize};
use windows::{
    Win32::{
        Foundation::{CloseHandle, HANDLE},
        System::{
            Com::CoTaskMemFree,
            RemoteDesktop::{ProcessIdToSessionId, WTSQueryUserToken},
            Threading::GetCurrentProcessId,
        },
        UI::Shell::{FOLDERID_RoamingAppData, KNOWN_FOLDER_FLAG, SHGetKnownFolderPath},
    },
    core::PWSTR,
};
use winsafe::{HWND, co, prelude::Handle};

use super::AppLanguage;

/// The outcome of attempting to read and parse the config file.
///
/// Used internally by [`AppConfig::load`] to separate the three
/// distinct outcomes without collapsing them into a single error type.
pub enum ConfigLoadResult {
    /// The file was found and parsed successfully.
    Loaded(AppConfig),
    /// The file does not exist. A default config should be created.
    NotFound(AppConfig),
    /// The file exists but could not be read or parsed.
    ParseFailed(anyhow::Error),
}

/// The persisted application configuration.
///
/// Serialized to TOML with uppercase keys, e.g.:
/// ```toml
/// LANGUAGE = "en-US"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE", deny_unknown_fields)]
pub struct AppConfig {
    pub language: AppLanguage,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: AppLanguage::default(),
        }
    }
}

impl AppConfig {
    /// Returns the expected path to the config file for the active interactive user.
    ///
    /// When running under elevated system tokens such as TrustedInstaller, queries
    /// the current desktop session token to correctly resolve the physical or remote
    /// logged-in user's `%APPDATA%` directory instead of `systemprofile`. Falls back
    /// to [`dirs::config_dir`] if session queries fail.
    pub fn config_path() -> Option<PathBuf> {
        Self::resolve_active_user_appdata_dir()
            .or_else(dirs::config_dir)
            .map(|base_config_directory| {
                base_config_directory
                    .join(env!("CARGO_PKG_NAME"))
                    .join("CONFIG.toml")
            })
    }

    /// Resolve the Roaming AppData directory corresponding to the logged-in interactive user.
    fn resolve_active_user_appdata_dir() -> Option<PathBuf> {
        let current_session_id = fetch_current_process_session_id()?;
        let user_token_guard = query_session_user_token(current_session_id)?;
        fetch_roaming_appdata_by_token(user_token_guard.as_raw())
    }

    /// Load the config from disk, handling all error cases gracefully.
    ///
    /// Never returns an error — all failure modes are handled internally
    /// by falling back to the default config and showing an error dialog
    /// where appropriate.
    pub fn load() -> Self {
        match Self::load_config_content() {
            ConfigLoadResult::Loaded(config) => config,
            ConfigLoadResult::NotFound(default_config) => {
                Self::handle_missing_config(default_config)
            }
            ConfigLoadResult::ParseFailed(error) => Self::handle_corrupted_config(error),
        }
    }

    /// Attempt to read and parse the config file, returning a typed outcome.
    fn load_config_content() -> ConfigLoadResult {
        let config_path = match Self::config_path() {
            Some(config_path) => config_path,
            None => return ConfigLoadResult::NotFound(Self::default()),
        };

        if !config_path.exists() {
            return ConfigLoadResult::NotFound(Self::default());
        }

        let toml_content = match Self::read_config(&config_path) {
            Ok(content) => content,
            Err(read_error) => return ConfigLoadResult::ParseFailed(read_error),
        };

        match Self::parse_config(&toml_content) {
            Ok(config) => ConfigLoadResult::Loaded(config),
            Err(parse_error) => ConfigLoadResult::ParseFailed(parse_error),
        }
    }

    /// Read the config file contents from disk.
    fn read_config(config_path: &PathBuf) -> Result<String> {
        fs::read_to_string(config_path).with_context(|| {
            t!(
                "CONFIG_READ_FAILED",
                config_path = config_path.display().to_string()
            )
        })
    }

    /// Parse a TOML string into an [`AppConfig`].
    fn parse_config(toml_content: &str) -> Result<AppConfig> {
        toml::from_str::<AppConfig>(toml_content).context(t!("CONFIG_PARSE_FAILED"))
    }

    /// Save the default config to disk when no config file exists.
    ///
    /// Panics if the save fails, because there is no safe way to continue
    /// without a writable config directory.
    fn handle_missing_config(default_config: Self) -> Self {
        if let Err(save_error) = default_config.save() {
            panic!(
                "{}",
                t!("CONFIG_SAVE_DEFAULT_FAILED", save_error = save_error)
            );
        }
        default_config
    }

    /// Show an error dialog for a corrupt config, overwrite it with defaults,
    /// and return the default config.
    ///
    /// Panics if the subsequent save also fails.
    fn handle_corrupted_config(error: anyhow::Error) -> Self {
        Self::show_error_dialog(&t!(
            "CONFIG_PARSE_FAILED_USING_DEFAULT",
            parse_error = error
        ));

        let default_config = Self::default();
        if let Err(save_error) = default_config.save() {
            panic!(
                "{}",
                t!("CONFIG_SAVE_DEFAULT_FAILED", save_error = save_error)
            );
        }
        default_config
    }

    /// Display an error dialog using the null HWND (no parent window).
    fn show_error_dialog(message: &str) {
        HWND::NULL
            .MessageBox(message, &t!("ERROR"), co::MB::OK | co::MB::ICONERROR)
            .ok();
    }

    /// Serialize this config to TOML and write it to the config file path.
    ///
    /// Creates the parent directory if it does not already exist.
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path().context(t!("CONFIG_DIR_NOT_FOUND"))?;

        if let Some(config_path_parent) = config_path.parent() {
            fs::create_dir_all(config_path_parent).with_context(|| {
                t!(
                    "CONFIG_DIR_CREATE_FAILED",
                    config_path_parent = config_path_parent.display().to_string()
                )
            })?;
        }

        let toml_content = toml::to_string_pretty(self).context(t!("CONFIG_SERIALIZE_FAILED"))?;
        fs::write(&config_path, toml_content).with_context(|| {
            t!(
                "CONFIG_WRITE_FAILED",
                config_path = config_path.display().to_string()
            )
        })?;

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Active User Session Path Helpers
// ---------------------------------------------------------------------------

/// RAII guard for a Win32 [`HANDLE`] ensuring automatic closure on drop.
struct HandleGuard {
    raw_handle: HANDLE,
}

impl HandleGuard {
    /// Create a new handle guard wrapper.
    fn new(raw_handle: HANDLE) -> Self {
        Self { raw_handle }
    }

    /// Access the underlying raw Win32 [`HANDLE`].
    fn as_raw(&self) -> HANDLE {
        self.raw_handle
    }
}

impl Drop for HandleGuard {
    fn drop(&mut self) {
        if !self.raw_handle.is_invalid() {
            unsafe {
                let _ = CloseHandle(self.raw_handle);
            }
        }
    }
}

/// RAII guard for a COM-allocated [`PWSTR`] buffer ensuring memory is freed on drop.
struct CoTaskMemGuard {
    raw_pwstr: PWSTR,
}

impl CoTaskMemGuard {
    /// Create a new COM task memory guard wrapper.
    fn new(raw_pwstr: PWSTR) -> Self {
        Self { raw_pwstr }
    }

    /// Directly convert the UTF-16 buffer to a native Windows [`PathBuf`].
    fn to_path_buf(&self) -> PathBuf {
        let utf16_slice = unsafe { self.raw_pwstr.as_wide() };
        PathBuf::from(OsString::from_wide(utf16_slice))
    }
}

impl Drop for CoTaskMemGuard {
    fn drop(&mut self) {
        if !self.raw_pwstr.is_null() {
            unsafe {
                CoTaskMemFree(Some(self.raw_pwstr.0.cast()));
            }
        }
    }
}

/// Retrieve the session identifier of the current GUI process.
fn fetch_current_process_session_id() -> Option<u32> {
    let mut current_process_session_id = 0;
    let query_session_result =
        unsafe { ProcessIdToSessionId(GetCurrentProcessId(), &mut current_process_session_id) };

    if query_session_result.is_ok() {
        Some(current_process_session_id)
    } else {
        None
    }
}

/// Acquire the primary user token associated with the given session ID.
fn query_session_user_token(session_id: u32) -> Option<HandleGuard> {
    let mut user_token = HANDLE::default();
    let query_token_result = unsafe { WTSQueryUserToken(session_id, &mut user_token) };

    if query_token_result.is_ok() && !user_token.is_invalid() {
        Some(HandleGuard::new(user_token))
    } else {
        None
    }
}

/// Query the roaming AppData path for the user identified by the specified token.
fn fetch_roaming_appdata_by_token(user_token: HANDLE) -> Option<PathBuf> {
    let folder_path_result =
        unsafe { SHGetKnownFolderPath(&FOLDERID_RoamingAppData, KNOWN_FOLDER_FLAG(0), user_token) };

    let path_pwstr = match folder_path_result {
        Ok(path_pwstr) if !path_pwstr.is_null() => path_pwstr,
        _ => return None,
    };

    let memory_guard = CoTaskMemGuard::new(path_pwstr);
    Some(memory_guard.to_path_buf())
}
