//! Desktop visual metrics, colors, and classic scheme registry operations.
//!
//! Provides routines to clone original non-client metrics, colors, and visual
//! appearance configurations from the offline template hive into the active
//! interactive user's registry hive, as well as installing extra classic schemes.

use std::borrow::Cow;

use anyhow::{Context, Result, bail};
use rust_i18n::t;
use winreg::{
    RegKey, RegValue,
    enums::{HKEY_USERS, KEY_ALL_ACCESS, KEY_READ, RegType},
};

use crate::desktop::{
    session,
    theme::{
        hive::{
            LoadedHiveGuard, TEMPORARY_DEFAULT_HIVE_SUBKEY_NAME, resolve_default_user_hive_path,
        },
        schemes,
    },
};

/// Relative subkey paths within the user profile hive that must be cloned for metrics restoration.
const TARGET_METRICS_REGISTRY_SUBKEY_PATHS: &[&str] = &[
    r"Control Panel\Appearance",
    r"Control Panel\Colors",
    r"Control Panel\Desktop\WindowMetrics",
];

/// Relative subkey path within the user profile hive containing classic visual appearance schemes.
const APPEARANCE_SCHEMES_SUBKEY_PATH: &str = r"Control Panel\Appearance\Schemes";

/// Restore default visual styles, system colors, and non-client metrics for the active user.
///
/// Copies `Default\NTUSER.DAT` to a temporary staging directory, mounts the replica into
/// `HKEY_USERS`, purges existing configurations under the target keys, clones default entries
/// into the active user's registry hive, and ensures clean unmounting on completion.
pub fn apply_default_metrics() -> Result<()> {
    let default_hive_file_path = resolve_default_user_hive_path()?;
    let default_hive_file_path_string = default_hive_file_path.display().to_string();
    if !default_hive_file_path.exists() {
        bail!(
            "{}",
            t!(
                "ERROR_DEFAULT_HIVE_NOT_FOUND",
                default_hive_file_path = default_hive_file_path_string
            )
        );
    }

    // 1. Copy and mount the offline Default NTUSER.DAT into HKEY_USERS
    let _loaded_hive_guard =
        LoadedHiveGuard::mount(TEMPORARY_DEFAULT_HIVE_SUBKEY_NAME, &default_hive_file_path)
            .with_context(|| {
                format!(
                    "{}: {}",
                    t!(
                        "ERROR_MOUNT_DEFAULT_HIVE_FAILED",
                        default_hive_file_path = default_hive_file_path_string
                    ),
                    default_hive_file_path.display()
                )
            })?;

    // 2. Open registry roots. Because source_template_root_key and target_user_root_key
    // are declared after _loaded_hive_guard, Rust's LIFO drop order guarantees that
    // all open registry subkey handles are closed before _loaded_hive_guard calls RegUnLoadKey.
    let users_root_key = RegKey::predef(HKEY_USERS);
    let source_template_root_key = users_root_key
        .open_subkey_with_flags(TEMPORARY_DEFAULT_HIVE_SUBKEY_NAME, KEY_READ)
        .context(t!("ERROR_OPEN_MOUNTED_HIVE_FAILED"))?;

    let target_user_root_key = session::open_active_user_registry_root()?;

    // 3. Purge existing target keys and clone fresh contents
    for relative_metrics_subkey_path in TARGET_METRICS_REGISTRY_SUBKEY_PATHS {
        reset_active_user_subkey_from_mounted_template(
            &source_template_root_key,
            &target_user_root_key,
            relative_metrics_subkey_path,
        )?;
    }

    Ok(())
}

/// Restore default classic visual appearance schemes for the active user.
///
/// Copies `Default\NTUSER.DAT` to a temporary directory, mounts the replica into `HKEY_USERS`,
/// purges existing configurations under `Control Panel\Appearance\Schemes`, clones default preset
/// schemes into the active user's registry hive, and cleanly unmounts the temporary hive.
pub fn restore_default_classic_schemes() -> Result<()> {
    let default_hive_file_path = resolve_default_user_hive_path()?;
    let default_hive_file_path_string = default_hive_file_path.display().to_string();
    if !default_hive_file_path.exists() {
        bail!(
            "{}",
            t!(
                "ERROR_DEFAULT_HIVE_NOT_FOUND",
                default_hive_file_path = default_hive_file_path_string
            )
        );
    }

    let _loaded_hive_guard =
        LoadedHiveGuard::mount(TEMPORARY_DEFAULT_HIVE_SUBKEY_NAME, &default_hive_file_path)
            .with_context(|| {
                format!(
                    "{}: {}",
                    t!(
                        "ERROR_MOUNT_DEFAULT_HIVE_FAILED",
                        default_hive_file_path = default_hive_file_path_string
                    ),
                    default_hive_file_path.display()
                )
            })?;

    let users_root_key = RegKey::predef(HKEY_USERS);
    let source_template_root_key = users_root_key
        .open_subkey_with_flags(TEMPORARY_DEFAULT_HIVE_SUBKEY_NAME, KEY_READ)
        .context(t!("ERROR_OPEN_MOUNTED_HIVE_FAILED"))?;

    let target_user_root_key = session::open_active_user_registry_root()?;

    reset_active_user_subkey_from_mounted_template(
        &source_template_root_key,
        &target_user_root_key,
        APPEARANCE_SCHEMES_SUBKEY_PATH,
    )?;

    Ok(())
}

/// Purge existing user configuration and clone fresh values from the mounted default user hive subkey.
fn reset_active_user_subkey_from_mounted_template(
    source_template_root_key: &RegKey,
    target_user_root_key: &RegKey,
    relative_target_subkey_path: &str,
) -> Result<()> {
    let source_template_subkey = source_template_root_key
        .open_subkey_with_flags(relative_target_subkey_path, KEY_READ)
        .with_context(|| {
            t!(
                "ERROR_OPEN_DEFAULT_HIVE_SUBKEY_FAILED",
                subkey_path = relative_target_subkey_path
            )
        })?;

    let (target_user_subkey, _disposition) = target_user_root_key
        .create_subkey_with_flags(relative_target_subkey_path, KEY_ALL_ACCESS)
        .with_context(|| {
            t!(
                "ERROR_OPEN_ACTIVE_USER_SUBKEY_FAILED",
                subkey_path = relative_target_subkey_path
            )
        })?;

    // Step 1: Collect and delete all existing values under the target user's subkey
    let target_user_existing_value_names: Vec<String> = target_user_subkey
        .enum_values()
        .filter_map(|registry_value_entry_result| {
            registry_value_entry_result
                .ok()
                .map(|(existing_value_name, _existing_value_data)| existing_value_name)
        })
        .collect();

    for target_user_existing_value_name in target_user_existing_value_names {
        target_user_subkey
            .delete_value(&target_user_existing_value_name)
            .with_context(|| {
                t!(
                    "ERROR_REGISTRY_DELETE_VALUE_FAILED",
                    value_name = target_user_existing_value_name
                )
            })?;
    }

    // Step 2: Clone all values from the source template subkey
    for enum_result in source_template_subkey.enum_values() {
        let (source_value_name, source_value_data) = enum_result.with_context(|| {
            t!(
                "ERROR_REGISTRY_ENUM_VALUES_FAILED",
                subkey_path = relative_target_subkey_path
            )
        })?;

        target_user_subkey
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

/// Add extra classic visual appearance schemes to the active user's registry hive.
///
/// Writes predefined schemes directly to `Control Panel\Appearance\Schemes`
/// in the active interactive user's hive.
pub fn add_extra_classic_schemes() -> Result<()> {
    let target_user_root_key = session::open_active_user_registry_root()?;
    let (schemes_target_subkey, _disposition) = target_user_root_key
        .create_subkey_with_flags(APPEARANCE_SCHEMES_SUBKEY_PATH, KEY_ALL_ACCESS)
        .with_context(|| {
            t!(
                "ERROR_OPEN_ACTIVE_USER_SUBKEY_FAILED",
                subkey_path = APPEARANCE_SCHEMES_SUBKEY_PATH
            )
        })?;

    for &(scheme_target_value_name, scheme_payload_data) in schemes::get_extra_classic_schemes() {
        let registry_value = RegValue {
            vtype: RegType::REG_BINARY,
            bytes: Cow::Borrowed(scheme_payload_data),
        };

        schemes_target_subkey
            .set_raw_value(scheme_target_value_name, &registry_value)
            .with_context(|| {
                t!(
                    "ERROR_REGISTRY_SET_VALUE_FAILED",
                    value_name = scheme_target_value_name
                )
            })?;
    }

    Ok(())
}
