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

/// Safely execute an operation against the mounted default template registry hive and active user hive.
///
/// Discovers the system `Default\NTUSER.DAT`, mounts it into `HKEY_USERS` within an RAII guard,
/// opens both the template root and the interactive user's root, and executes the provided closure.
///
/// # Lifetime and Handle Safety
/// The closure operates inside an isolated lexical block so that all derived `RegKey` handles
/// and borrow guards are dropped before [`LoadedHiveGuard`] executes `RegUnLoadKey`, strictly
/// eliminating the risk of `ERROR_SHARING_VIOLATION` during hive unloading.
fn with_mounted_template_hive<T>(
    operation: impl FnOnce(&RegKey, &RegKey) -> Result<T>,
) -> Result<T> {
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

    // Inner scope guarantees open registry handles are closed prior to LoadedHiveGuard drop.
    let operation_result = {
        let users_root_key = RegKey::predef(HKEY_USERS);
        let source_template_root_key = users_root_key
            .open_subkey_with_flags(TEMPORARY_DEFAULT_HIVE_SUBKEY_NAME, KEY_READ)
            .context(t!("ERROR_OPEN_MOUNTED_HIVE_FAILED"))?;

        let target_user_root_key = session::open_active_user_registry_root()?;

        operation(&source_template_root_key, &target_user_root_key)
    };

    operation_result
}

/// Reset a series of subkeys in the active user's registry hive by cloning them from the mounted template.
fn reset_active_user_subkeys_from_template(relative_target_subkey_paths: &[&str]) -> Result<()> {
    with_mounted_template_hive(
        |source_template_root_key, target_user_root_key| -> Result<()> {
            for relative_target_subkey_path in relative_target_subkey_paths {
                reset_active_user_subkey_from_mounted_template(
                    source_template_root_key,
                    target_user_root_key,
                    relative_target_subkey_path,
                )?;
            }
            Ok(())
        },
    )
}

/// Restore default visual styles, system colors, and non-client metrics for the active user.
///
/// Mounts the default template hive, purges existing metrics and appearance settings in the
/// interactive user's profile, clones the original configurations, and cleanly unmounts the hive.
pub fn apply_default_metrics() -> Result<()> {
    reset_active_user_subkeys_from_template(TARGET_METRICS_REGISTRY_SUBKEY_PATHS)
}

/// Restore default classic visual appearance schemes for the active user.
///
/// Mounts the default template hive, purges existing entries under `Control Panel\Appearance\Schemes`,
/// clones default preset schemes into the active user's hive, and cleanly unmounts the hive.
pub fn restore_default_classic_schemes() -> Result<()> {
    reset_active_user_subkeys_from_template(&[APPEARANCE_SCHEMES_SUBKEY_PATH])
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
        let scheme_target_value_data = RegValue {
            vtype: RegType::REG_BINARY,
            bytes: Cow::Borrowed(scheme_payload_data),
        };

        schemes_target_subkey
            .set_raw_value(scheme_target_value_name, &scheme_target_value_data)
            .with_context(|| {
                t!(
                    "ERROR_REGISTRY_SET_VALUE_FAILED",
                    value_name = scheme_target_value_name
                )
            })?;
    }

    Ok(())
}
