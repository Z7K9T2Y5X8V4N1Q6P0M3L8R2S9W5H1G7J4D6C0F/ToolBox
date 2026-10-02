//! Generic registry import engine for desktop visual styles and metrics.
//!
//! Loads the default user profile registry hive (`Default\NTUSER.DAT`) to extract
//! original non-client metrics, colors, and visual appearance configurations
//! directly from the operating system template, ensuring correct localization across languages.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use rust_i18n::t;
use windows::{
    Win32::{
        Foundation::{CloseHandle, HANDLE, LUID},
        Security::{
            AdjustTokenPrivileges, LUID_AND_ATTRIBUTES, LookupPrivilegeValueW,
            SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY,
        },
        System::Threading::{GetCurrentProcess, OpenProcessToken},
    },
    core::{PCWSTR, w},
};
use winreg::{
    RegKey,
    enums::{HKEY_LOCAL_MACHINE, HKEY_USERS, KEY_ALL_ACCESS, KEY_READ},
};
use winsafe::{ExpandEnvironmentStrings, HKEY, co};

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

/// RAII guard ensuring the temporary mounted registry hive is properly unloaded on drop.
struct LoadedHiveGuard {
    mount_subkey_name: String,
}

impl LoadedHiveGuard {
    /// Mount the offline hive file located at `hive_file_path` under `HKEY_USERS\<mount_subkey_name>`.
    fn load(mount_subkey_name: &str, hive_file_path: &Path) -> Result<Self> {
        enable_required_registry_privileges()
            .context(t!("ERROR_ENABLE_REGISTRY_PRIVILEGES_FAILED"))?;

        let hive_path_string = hive_file_path
            .to_str()
            .context(t!("ERROR_RESOLVE_DEFAULT_PROFILE_PATH_FAILED"))?;

        HKEY::USERS
            .RegLoadKey(Some(mount_subkey_name), hive_path_string)
            .map_err(|error| anyhow!("{error}"))?;

        Ok(Self {
            mount_subkey_name: mount_subkey_name.to_string(),
        })
    }
}

impl Drop for LoadedHiveGuard {
    fn drop(&mut self) {
        if let Err(unload_error) = HKEY::USERS.RegUnLoadKey(Some(&self.mount_subkey_name)) {
            log::warn!(
                "{}",
                t!(
                    "WARN_UNLOAD_DEFAULT_HIVE_FAILED",
                    subkey = self.mount_subkey_name,
                    error = unload_error
                )
            );
        }
    }
}

/// Restore default visual styles, system colors, and non-client metrics for the active user.
///
/// Mounts `Default\NTUSER.DAT`, purges existing configuration under the target keys,
/// clones default entries into the active user's registry hive, and unloads the file when complete.
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

    // 1. Mount the offline Default NTUSER.DAT into HKEY_USERS
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

    // 2. Open both the mounted hive and the active user registry root
    let root_users_key = RegKey::predef(HKEY_USERS);
    let mounted_default_key = root_users_key
        .open_subkey_with_flags(TEMPORARY_DEFAULT_HIVE_SUBKEY, KEY_READ)
        .context(t!("ERROR_OPEN_MOUNTED_HIVE_FAILED"))?;

    let active_user_key = session::open_active_user_registry_root()?;

    // 3. Purge existing target keys and clone fresh contents (Scheme A: Full replacement)
    for target_theme_metrics_branch_path in TARGET_REGISTRY_SUBKEYS {
        reset_target_metrics_branch_from_default_hive(
            &mounted_default_key,
            &active_user_key,
            target_theme_metrics_branch_path,
        )?;
    }

    Ok(())
}

/// Purge existing user configuration and clone fresh metric values from the mounted default user hive.
fn reset_target_metrics_branch_from_default_hive(
    mounted_default_user_root_key: &RegKey,
    active_user_registry_root_key: &RegKey,
    target_theme_metrics_branch_path: &str,
) -> Result<()> {
    let source_default_metrics_subkey = mounted_default_user_root_key
        .open_subkey_with_flags(target_theme_metrics_branch_path, KEY_READ)
        .with_context(|| {
            t!(
                "ERROR_OPEN_DEFAULT_HIVE_SUBKEY_FAILED",
                subkey_path = target_theme_metrics_branch_path
            )
        })?;

    let active_user_metrics_subkey = active_user_registry_root_key
        .open_subkey_with_flags(target_theme_metrics_branch_path, KEY_ALL_ACCESS)
        .with_context(|| {
            t!(
                "ERROR_OPEN_ACTIVE_USER_SUBKEY_FAILED",
                subkey_path = target_theme_metrics_branch_path
            )
        })?;

    // Step 1: Collect and delete all existing values under the active user's target branch
    let old_value_names: Vec<String> = active_user_metrics_subkey
        .enum_values()
        .filter_map(|enum_result| enum_result.ok().map(|(value_name, _)| value_name))
        .collect();

    for old_value_name in old_value_names {
        active_user_metrics_subkey
            .delete_value(&old_value_name)
            .with_context(|| {
                t!(
                    "ERROR_REGISTRY_DELETE_VALUE_FAILED",
                    value_name = old_value_name
                )
            })?;
    }

    // Step 2: Clone all pristine values from the default user template branch
    for enum_result in source_default_metrics_subkey.enum_values() {
        let (pristine_value_name, pristine_value_data) = enum_result.with_context(|| {
            t!(
                "ERROR_REGISTRY_ENUM_VALUES_FAILED",
                subkey_path = target_theme_metrics_branch_path
            )
        })?;

        active_user_metrics_subkey
            .set_raw_value(&pristine_value_name, &pristine_value_data)
            .with_context(|| {
                t!(
                    "ERROR_REGISTRY_SET_VALUE_FAILED",
                    value_name = pristine_value_name
                )
            })?;
    }

    Ok(())
}

/// Enable `SeBackupPrivilege` and `SeRestorePrivilege` for the current process token.
fn enable_required_registry_privileges() -> Result<()> {
    struct TokenHandleGuard(HANDLE);
    impl Drop for TokenHandleGuard {
        fn drop(&mut self) {
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }

    let mut token_handle = HANDLE::default();
    unsafe {
        OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token_handle,
        )?;
    }

    let token_handle_guard = TokenHandleGuard(token_handle);

    for privilege_name in [w!("SeBackupPrivilege"), w!("SeRestorePrivilege")] {
        enable_single_privilege(token_handle_guard.0, privilege_name)?;
    }

    Ok(())
}

/// Enable a single privilege on the specified token.
fn enable_single_privilege(token_handle: HANDLE, privilege_name: PCWSTR) -> Result<()> {
    let mut luid = LUID::default();
    unsafe {
        LookupPrivilegeValueW(None, privilege_name, &mut luid)?;
    }

    let token_privileges = TOKEN_PRIVILEGES {
        PrivilegeCount: 1,
        Privileges: [LUID_AND_ATTRIBUTES {
            Luid: luid,
            Attributes: SE_PRIVILEGE_ENABLED,
        }],
    };

    unsafe {
        AdjustTokenPrivileges(token_handle, false, Some(&token_privileges), 0, None, None)?;
    }

    Ok(())
}

/// Resolve the absolute path to the default user's `NTUSER.DAT` hive file dynamically.
fn resolve_default_user_hive_path() -> Result<PathBuf> {
    // 1. Try Windows Known Folder (UserProfiles, e.g. D:\Users)
    if let Some(user_profiles_dir) = fetch_user_profiles_known_folder() {
        let candidate_path = user_profiles_dir.join("Default").join("NTUSER.DAT");
        if candidate_path.exists() {
            return Ok(candidate_path);
        }
    }

    // 2. Try HKLM ProfileList registry entry (expanded if contains environment variables)
    if let Ok(registry_default_path) = query_default_profile_path_from_registry() {
        let candidate_path = registry_default_path.join("NTUSER.DAT");
        if candidate_path.exists() {
            return Ok(candidate_path);
        }
    }

    // 3. Fallback: Dynamically resolve from active %SystemDrive% (never hardcode "C:\")
    if let Some(system_drive) = std::env::var_os("SystemDrive") {
        let candidate_path = PathBuf::from(system_drive)
            .join(std::path::MAIN_SEPARATOR_STR)
            .join("Users")
            .join("Default")
            .join("NTUSER.DAT");

        if candidate_path.exists() {
            return Ok(candidate_path);
        }
    }

    bail!("{}", t!("ERROR_RESOLVE_DEFAULT_PROFILE_PATH_FAILED"))
}

/// Query the path to the user profiles root directory using [`winsafe::SHGetKnownFolderPath`].
fn fetch_user_profiles_known_folder() -> Option<PathBuf> {
    winsafe::SHGetKnownFolderPath(&co::KNOWNFOLDERID::UserProfiles, co::KF::DEFAULT, None)
        .ok()
        .map(PathBuf::from)
}

/// Query the default user profile path configured under `ProfileList` in HKLM.
fn query_default_profile_path_from_registry() -> Result<PathBuf> {
    let local_machine_root = RegKey::predef(HKEY_LOCAL_MACHINE);
    let profile_list_key = local_machine_root
        .open_subkey_with_flags(PROFILE_LIST_REGISTRY_PATH, KEY_READ)
        .context(t!("ERROR_OPEN_PROFILE_LIST_KEY_FAILED"))?;

    let default_profile_raw: String = profile_list_key
        .get_value(DEFAULT_PROFILE_VALUE_NAME)
        .context(t!("ERROR_READ_DEFAULT_PROFILE_VALUE_FAILED"))?;

    // Expand potential environment variables like %SystemDrive%
    let expanded_path =
        ExpandEnvironmentStrings(&default_profile_raw).unwrap_or(default_profile_raw);

    Ok(PathBuf::from(expanded_path))
}
