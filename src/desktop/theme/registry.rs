//! Generic registry import engine for desktop visual styles and metrics.
//!
//! Copies the default user profile registry hive (`Default\NTUSER.DAT`) to an isolated
//! temporary directory, loads it into a temporary subkey, extracts original non-client
//! metrics, colors, and visual appearance configurations directly from the operating
//! system template, and restores them to the active interactive user's registry hive.

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow, bail};
use elevate_ti::{Privilege, ProcessToken};
use rust_i18n::t;
use windows::Win32::System::Threading::GetCurrentProcessId;
use winreg::{
    RegKey,
    enums::{HKEY_LOCAL_MACHINE, HKEY_USERS, KEY_ALL_ACCESS, KEY_READ},
};
use winsafe::{ExpandEnvironmentStrings, HKEY, SHGetKnownFolderPath, co};

use crate::desktop::session;

/// Unique subkey name under `HKEY_USERS` used for mounting the default user profile hive temporarily.
const TEMPORARY_DEFAULT_HIVE_SUBKEY: &str =
    "Z7K9T2Y5X8V4N1Q6P0M3L8R2S9W5H1G7J4D6C0F.Toolbox.TemporaryDefaultUserHiveMount";

/// Relative paths within the user profile hive that must be cloned.
const TARGET_REGISTRY_SUBKEYS: &[&str] = &[
    r"Control Panel\Appearance",
    r"Control Panel\Colors",
    r"Control Panel\Desktop\WindowMetrics",
];

/// Registry path holding profile directory configurations.
const PROFILE_LIST_REGISTRY_PATH: &str =
    r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\ProfileList";

/// Registry value name for the default user profile directory.
const DEFAULT_PROFILE_VALUE_NAME: &str = "Default";

/// File name of the user profile registry hive.
const DEFAULT_USER_PROFILE_HIVE_FILE_NAME: &str = "NTUSER.DAT";

/// RAII guard ensuring the temporary mounted registry hive is properly unloaded
/// and its isolated temporary directory is cleaned up on drop.
struct LoadedHiveGuard {
    mounted_hive_subkey_name: String,
    temporary_directory_path: PathBuf,
}

impl LoadedHiveGuard {
    /// Copy the offline hive file to an isolated temporary location and mount it under `HKEY_USERS`.
    fn load(mounted_hive_subkey_name: &str, source_hive_file_path: &Path) -> Result<Self> {
        enable_required_registry_privileges()
            .context(t!("ERROR_ENABLE_REGISTRY_PRIVILEGES_FAILED"))?;

        let current_process_id = unsafe { GetCurrentProcessId() };
        let timestamp_millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(1_145_140_000_000, |duration| duration.as_millis());

        let temporary_directory_name =
            format!("{TEMPORARY_DEFAULT_HIVE_SUBKEY}.{current_process_id}{timestamp_millis}");
        let temporary_directory_path = std::env::temp_dir().join(temporary_directory_name);

        fs::create_dir_all(&temporary_directory_path).context(t!(
            "CONFIG_DIR_CREATE_FAILED",
            config_path_parent = temporary_directory_path.display().to_string()
        ))?;

        let temporary_hive_file_path =
            temporary_directory_path.join(DEFAULT_USER_PROFILE_HIVE_FILE_NAME);
        fs::copy(source_hive_file_path, &temporary_hive_file_path).context(t!(
            "CONFIG_WRITE_FAILED",
            config_path = temporary_hive_file_path.display().to_string()
        ))?;

        let hive_path_string = temporary_hive_file_path
            .to_str()
            .context(t!("ERROR_RESOLVE_DEFAULT_PROFILE_PATH_FAILED"))?;

        HKEY::USERS
            .RegLoadKey(Some(mounted_hive_subkey_name), hive_path_string)
            .map_err(|error| anyhow!("{error}"))?;

        Ok(Self {
            mounted_hive_subkey_name: mounted_hive_subkey_name.to_string(),
            temporary_directory_path,
        })
    }
}

impl Drop for LoadedHiveGuard {
    fn drop(&mut self) {
        if let Err(unload_error) = HKEY::USERS.RegUnLoadKey(Some(&self.mounted_hive_subkey_name)) {
            let unload_error_description = unload_error.to_string();
            log::warn!(
                "{}",
                t!(
                    "WARN_UNLOAD_DEFAULT_HIVE_FAILED",
                    subkey = self.mounted_hive_subkey_name,
                    error = unload_error_description
                )
            );
        }

        let _ = fs::remove_dir_all(&self.temporary_directory_path);
    }
}

/// Restore default visual styles, system colors, and non-client metrics for the active user.
///
/// Copies `Default\NTUSER.DAT` to a temporary directory, mounts the replica into `HKEY_USERS`,
/// purges existing configurations under the target keys, clones default entries into the active
/// user's registry hive, and ensures clean unmounting and removal of temporary files.
pub fn apply_default_metrics() -> Result<()> {
    let default_hive_file_path = resolve_default_user_hive_path()?;
    if !default_hive_file_path.exists() {
        bail!(
            "{}",
            t!(
                "ERROR_DEFAULT_HIVE_NOT_FOUND",
                path = default_hive_file_path.display().to_string()
            )
        );
    }

    // 1. Copy and mount the offline Default NTUSER.DAT into HKEY_USERS
    let _loaded_hive_guard =
        LoadedHiveGuard::load(TEMPORARY_DEFAULT_HIVE_SUBKEY, &default_hive_file_path)
            .with_context(|| {
                format!(
                    "{}: {}",
                    t!(
                        "ERROR_MOUNT_DEFAULT_HIVE_FAILED",
                        path = default_hive_file_path.display().to_string()
                    ),
                    default_hive_file_path.display()
                )
            })?;

    // 2. Open registry roots. Because mounted_template_root_key and active_user_root_key
    // are declared after _loaded_hive_guard, Rust's LIFO drop order guarantees that
    // all open registry subkey handles are closed before _loaded_hive_guard calls RegUnLoadKey.
    let users_root_key = RegKey::predef(HKEY_USERS);
    let mounted_template_root_key = users_root_key
        .open_subkey_with_flags(TEMPORARY_DEFAULT_HIVE_SUBKEY, KEY_READ)
        .context(t!("ERROR_OPEN_MOUNTED_HIVE_FAILED"))?;

    let active_user_root_key = session::open_active_user_registry_root()?;

    // 3. Purge existing target keys and clone fresh contents
    for relative_metrics_subkey_path in TARGET_REGISTRY_SUBKEYS {
        reset_active_user_metrics_subkey_from_mounted_template(
            &mounted_template_root_key,
            &active_user_root_key,
            relative_metrics_subkey_path,
        )?;
    }

    Ok(())
}

/// Purge existing user configuration and clone fresh metric values from the mounted default user hive.
fn reset_active_user_metrics_subkey_from_mounted_template(
    mounted_template_root_key: &RegKey,
    active_user_root_key: &RegKey,
    relative_metrics_subkey_path: &str,
) -> Result<()> {
    let mounted_template_metrics_subkey = mounted_template_root_key
        .open_subkey_with_flags(relative_metrics_subkey_path, KEY_READ)
        .with_context(|| {
            t!(
                "ERROR_OPEN_DEFAULT_HIVE_SUBKEY_FAILED",
                subkey_path = relative_metrics_subkey_path
            )
        })?;

    let active_user_metrics_subkey = active_user_root_key
        .open_subkey_with_flags(relative_metrics_subkey_path, KEY_ALL_ACCESS)
        .with_context(|| {
            t!(
                "ERROR_OPEN_ACTIVE_USER_SUBKEY_FAILED",
                subkey_path = relative_metrics_subkey_path
            )
        })?;

    // Step 1: Collect and delete all existing values under the target user's metrics subkey
    let active_user_existing_metrics_value_names: Vec<String> = active_user_metrics_subkey
        .enum_values()
        .filter_map(|enum_result| enum_result.ok().map(|(value_name, _)| value_name))
        .collect();

    for active_user_existing_metrics_value_name in active_user_existing_metrics_value_names {
        active_user_metrics_subkey
            .delete_value(&active_user_existing_metrics_value_name)
            .with_context(|| {
                t!(
                    "ERROR_REGISTRY_DELETE_VALUE_FAILED",
                    value_name = active_user_existing_metrics_value_name
                )
            })?;
    }

    // Step 2: Clone all values from the source default template subkey
    for enum_result in mounted_template_metrics_subkey.enum_values() {
        let (source_value_name, source_value_data) = enum_result.with_context(|| {
            t!(
                "ERROR_REGISTRY_ENUM_VALUES_FAILED",
                subkey_path = relative_metrics_subkey_path
            )
        })?;

        active_user_metrics_subkey
            .set_raw_value(&source_value_name, &source_value_data)
            .with_context(|| {
                t!(
                    "ERROR_REGISTRY_SET_VALUE_FAILED",
                    value_name = source_value_name
                )
            })?;
    }

    Ok(())
}

/// Enable `SeBackupPrivilege` and `SeRestorePrivilege` for the current process token.
fn enable_required_registry_privileges() -> Result<()> {
    let current_token = ProcessToken::current_process()?;
    current_token.enable_privileges(&[Privilege::Backup, Privilege::Restore])?;
    Ok(())
}

/// Resolve the absolute path to the default user's `NTUSER.DAT` hive file dynamically.
fn resolve_default_user_hive_path() -> Result<PathBuf> {
    // 1. Try Windows Known Folder (UserProfiles, e.g. D:\Users)
    if let Some(user_profiles_directory) = fetch_user_profiles_known_folder() {
        let known_folder_candidate_path = user_profiles_directory
            .join("Default")
            .join(DEFAULT_USER_PROFILE_HIVE_FILE_NAME);
        if known_folder_candidate_path.exists() {
            return Ok(known_folder_candidate_path);
        }
    }

    // 2. Try HKLM ProfileList registry entry (expanded if contains environment variables)
    if let Ok(registry_default_profile_directory) = query_default_profile_path_from_registry() {
        let registry_candidate_path =
            registry_default_profile_directory.join(DEFAULT_USER_PROFILE_HIVE_FILE_NAME);
        if registry_candidate_path.exists() {
            return Ok(registry_candidate_path);
        }
    }

    // 3. Fallback: Dynamically resolve from active %SystemDrive% (never hardcode "C:\")
    if let Some(system_drive) = std::env::var_os("SystemDrive") {
        let system_drive_candidate_path = PathBuf::from(system_drive)
            .join(std::path::MAIN_SEPARATOR_STR)
            .join("Users")
            .join("Default")
            .join(DEFAULT_USER_PROFILE_HIVE_FILE_NAME);

        if system_drive_candidate_path.exists() {
            return Ok(system_drive_candidate_path);
        }
    }

    bail!("{}", t!("ERROR_RESOLVE_DEFAULT_PROFILE_PATH_FAILED"))
}

/// Query the path to the user profiles root directory using [`winsafe::SHGetKnownFolderPath`].
fn fetch_user_profiles_known_folder() -> Option<PathBuf> {
    SHGetKnownFolderPath(&co::KNOWNFOLDERID::UserProfiles, co::KF::DEFAULT, None)
        .ok()
        .map(PathBuf::from)
}

/// Query the default user profile path configured under `ProfileList` in HKLM.
fn query_default_profile_path_from_registry() -> Result<PathBuf> {
    let local_machine_root_key = RegKey::predef(HKEY_LOCAL_MACHINE);
    let profile_list_key = local_machine_root_key
        .open_subkey_with_flags(PROFILE_LIST_REGISTRY_PATH, KEY_READ)
        .context(t!("ERROR_OPEN_PROFILE_LIST_KEY_FAILED"))?;

    let default_profile_raw_string: String = profile_list_key
        .get_value(DEFAULT_PROFILE_VALUE_NAME)
        .context(t!("ERROR_READ_DEFAULT_PROFILE_VALUE_FAILED"))?;

    // Expand potential environment variables like %SystemDrive%
    let expanded_profile_path_string =
        ExpandEnvironmentStrings(&default_profile_raw_string).unwrap_or(default_profile_raw_string);

    Ok(PathBuf::from(expanded_profile_path_string))
}
