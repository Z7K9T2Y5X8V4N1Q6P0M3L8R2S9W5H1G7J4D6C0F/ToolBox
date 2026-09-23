//! Generic registry import engine for desktop visual styles and metrics.
//!
//! Provides routines to batch-write registry keys and values to the interactive
//! user hive with consistent error context propagation.

use anyhow::{Context, Result};
use rust_i18n::t;
use winreg::{RegKey, RegValue};

use super::presets::metrics;
use crate::desktop::session;

/// Restore default visual styles, system colors, and non-client metrics for the active user.
///
/// Writes the default presets into the interactive user's registry hive.
/// Note: Windows requires a user sign-out or system restart to reload these metrics.
pub fn apply_default_metrics() -> Result<()> {
    let user_root_key = session::open_active_user_registry_root()?;

    // 1. Write Control Panel\Appearance
    let appearance_key =
        create_user_subkey_with_context(&user_root_key, r"Control Panel\Appearance")?;
    write_string_entries(&appearance_key, &metrics::APPEARANCE_ENTRIES)?;

    // 2. Write Control Panel\Colors
    let colors_key = create_user_subkey_with_context(&user_root_key, r"Control Panel\Colors")?;
    write_string_entries(&colors_key, &metrics::COLOR_ENTRIES)?;

    // 3. Write Control Panel\Desktop\WindowMetrics
    let window_metrics_key =
        create_user_subkey_with_context(&user_root_key, r"Control Panel\Desktop\WindowMetrics")?;
    write_string_entries(&window_metrics_key, &metrics::WINDOW_METRICS_STRING_ENTRIES)?;
    write_uniform_raw_entries(
        &window_metrics_key,
        &metrics::WINDOW_METRICS_BINARY_FONT_KEYS,
        &metrics::default_font_reg_value(),
    )?;

    window_metrics_key
        .set_value("AppliedDPI", &metrics::APPLIED_DPI_VALUE)
        .with_context(|| t!("ERROR_REGISTRY_SET_VALUE_FAILED", value_name = "AppliedDPI"))?;

    Ok(())
}

/// Create or open a registry subkey and attach standardized error context.
fn create_user_subkey_with_context(parent_key: &RegKey, subkey_path: &str) -> Result<RegKey> {
    let (subkey, _) = parent_key.create_subkey(subkey_path).with_context(|| {
        t!(
            "ERROR_REGISTRY_OPEN_CREATE_KEY_FAILED",
            subkey_path = subkey_path
        )
    })?;
    Ok(subkey)
}

/// Batch-write a series of string key-value pairs into the specified registry key.
fn write_string_entries(target_key: &RegKey, entries: &[(&str, &str)]) -> Result<()> {
    for (value_name, value_data) in entries {
        target_key
            .set_value(value_name, value_data)
            .with_context(|| t!("ERROR_REGISTRY_SET_VALUE_FAILED", value_name = value_name))?;
    }
    Ok(())
}

/// Batch-write a uniform raw registry value across multiple value names.
fn write_uniform_raw_entries(
    target_key: &RegKey,
    value_names: &[&str],
    reg_value: &RegValue<'_>,
) -> Result<()> {
    for value_name in value_names {
        target_key
            .set_raw_value(value_name, reg_value)
            .with_context(|| t!("ERROR_REGISTRY_SET_VALUE_FAILED", value_name = value_name))?;
    }
    Ok(())
}
