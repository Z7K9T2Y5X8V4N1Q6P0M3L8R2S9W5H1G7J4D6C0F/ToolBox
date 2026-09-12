//! Config file load and save logic.
//!
//! The config file is stored at `{system config dir}/{app name}/CONFIG.toml`.
//! On Windows this resolves to `%APPDATA%\{app name}\CONFIG.toml`.
//!
//! # Load behavior
//! - If the file does not exist, a default config is written and returned.
//! - If the file exists but cannot be parsed, an error dialog is shown,
//!   the default config is written over the corrupt file, and returned.
//! - If the file exists and parses successfully, it is returned as-is.

use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use rust_i18n::t;
use serde::{Deserialize, Serialize};
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
    /// Returns the expected path to the config file, or `None` if the
    /// system config directory cannot be determined.
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|base_config_directory| {
            base_config_directory
                .join(env!("CARGO_PKG_NAME"))
                .join("CONFIG.toml")
        })
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
