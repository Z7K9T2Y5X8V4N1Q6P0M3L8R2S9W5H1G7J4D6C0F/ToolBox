//! Dynamic offline user profile hive resolution and mounting lifecycle management.
//!
//! Encapsulates the discovery of the default template hive (`Default\NTUSER.DAT`),
//! privilege elevation for offline hive operations, and an RAII guard that mounts
//! the replica into `HKEY_USERS` and ensures atomic unmounting and directory cleanup.

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
    enums::{HKEY_LOCAL_MACHINE, KEY_READ},
};
use winsafe::{ExpandEnvironmentStrings, HKEY, SHGetKnownFolderPath, co};

/// Unique subkey name under `HKEY_USERS` used for mounting the default user profile hive temporarily.
pub const TEMPORARY_DEFAULT_HIVE_SUBKEY_NAME: &str =
    "Z7K9T2Y5X8V4N1Q6P0M3L8R2S9W5H1G7J4D6C0F.Toolbox.TemporaryDefaultUserHiveMount";

/// Registry path holding user profile directory configurations.
const PROFILE_LIST_REGISTRY_PATH: &str =
    r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\ProfileList";

/// Registry value name for the default user profile directory.
const DEFAULT_PROFILE_VALUE_NAME: &str = "Default";

/// Standard filename of the user profile registry hive.
const DEFAULT_USER_PROFILE_HIVE_FILE_NAME: &str = "NTUSER.DAT";

/// RAII guard ensuring the temporarily mounted offline registry hive is cleanly
/// unloaded and its isolated temporary staging directory is deleted upon drop.
pub struct LoadedHiveGuard {
    mounted_hive_subkey_name: String,
    temporary_directory_path: PathBuf,
}

impl LoadedHiveGuard {
    /// Copy the offline hive file to an isolated staging directory and mount it under `HKEY_USERS`.
    pub fn mount(mounted_hive_subkey_name: &str, source_hive_file_path: &Path) -> Result<Self> {
        enable_required_registry_privileges()
            .context(t!("ERROR_ENABLE_REGISTRY_PRIVILEGES_FAILED"))?;

        let current_process_id = unsafe { GetCurrentProcessId() };
        let timestamp_millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(1_145_140_000_000, |duration| duration.as_millis());

        let temporary_directory_name =
            format!("{mounted_hive_subkey_name}.{current_process_id}{timestamp_millis}");
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
            .map_err(|hive_load_error| anyhow!("{hive_load_error}"))?;

        Ok(Self {
            mounted_hive_subkey_name: mounted_hive_subkey_name.to_string(),
            temporary_directory_path,
        })
    }
}

impl Drop for LoadedHiveGuard {
    fn drop(&mut self) {
        if let Err(hive_unload_error) =
            HKEY::USERS.RegUnLoadKey(Some(&self.mounted_hive_subkey_name))
        {
            let hive_unload_error_message = hive_unload_error.to_string();
            log::warn!(
                "{}",
                t!(
                    "WARN_UNLOAD_DEFAULT_HIVE_FAILED",
                    subkey = self.mounted_hive_subkey_name,
                    error_message = hive_unload_error_message
                )
            );
        }

        let _ = fs::remove_dir_all(&self.temporary_directory_path);
    }
}

/// Enable `SeBackupPrivilege` and `SeRestorePrivilege` on the current process token.
fn enable_required_registry_privileges() -> Result<()> {
    let current_token = ProcessToken::current_process()?;
    current_token.enable_privileges(&[Privilege::Backup, Privilege::Restore])?;
    Ok(())
}

/// Resolve the absolute path to the operating system default user's `NTUSER.DAT` hive file.
pub fn resolve_default_user_hive_path() -> Result<PathBuf> {
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

    let expanded_profile_path_string =
        ExpandEnvironmentStrings(&default_profile_raw_string).unwrap_or(default_profile_raw_string);

    Ok(PathBuf::from(expanded_profile_path_string))
}
