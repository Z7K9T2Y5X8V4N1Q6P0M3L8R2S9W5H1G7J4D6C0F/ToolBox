//! Generic registry import engine for desktop visual styles and metrics.

use anyhow::{Context, Result};
use rust_i18n::t;

use super::presets::metrics;
use crate::desktop::session;

/// Restore default visual styles, system colors, and non-client metrics for the active user.
///
/// Writes the default presets into the interactive user's registry hive.
/// Note: Windows requires a user sign-out or system restart to reload these metrics.
pub fn apply_default_metrics() -> Result<()> {
    let user_root_key = session::open_active_user_registry_root()?;

    // 1. Write Control Panel\Appearance
    let appearance_path = r"Control Panel\Appearance";
    let (appearance_key, _) = user_root_key
        .create_subkey(appearance_path)
        .with_context(|| {
            t!(
                "ERROR_REGISTRY_OPEN_CREATE_KEY_FAILED",
                subkey_path = appearance_path
            )
        })?;

    for (value_name, value_data) in metrics::APPEARANCE_ENTRIES {
        appearance_key
            .set_value(value_name, &value_data)
            .with_context(|| t!("ERROR_REGISTRY_SET_VALUE_FAILED", value_name = value_name))?;
    }

    // 2. Write Control Panel\Colors
    let colors_path = r"Control Panel\Colors";
    let (colors_key, _) = user_root_key.create_subkey(colors_path).with_context(|| {
        t!(
            "ERROR_REGISTRY_OPEN_CREATE_KEY_FAILED",
            subkey_path = colors_path
        )
    })?;

    for (value_name, value_data) in metrics::COLOR_ENTRIES {
        colors_key
            .set_value(value_name, &value_data)
            .with_context(|| t!("ERROR_REGISTRY_SET_VALUE_FAILED", value_name = value_name))?;
    }

    // 3. Write Control Panel\Desktop\WindowMetrics
    let window_metrics_path = r"Control Panel\Desktop\WindowMetrics";
    let (window_metrics_key, _) = user_root_key
        .create_subkey(window_metrics_path)
        .with_context(|| {
            t!(
                "ERROR_REGISTRY_OPEN_CREATE_KEY_FAILED",
                subkey_path = window_metrics_path
            )
        })?;

    for (value_name, value_data) in metrics::WINDOW_METRICS_STRING_ENTRIES {
        window_metrics_key
            .set_value(value_name, &value_data)
            .with_context(|| t!("ERROR_REGISTRY_SET_VALUE_FAILED", value_name = value_name))?;
    }

    let font_reg_value = metrics::default_font_reg_value();
    for font_key in metrics::WINDOW_METRICS_BINARY_FONT_KEYS {
        window_metrics_key
            .set_raw_value(font_key, &font_reg_value)
            .with_context(|| t!("ERROR_REGISTRY_SET_VALUE_FAILED", value_name = font_key))?;
    }

    window_metrics_key
        .set_value("AppliedDPI", &metrics::APPLIED_DPI_VALUE)
        .with_context(|| t!("ERROR_REGISTRY_SET_VALUE_FAILED", value_name = "AppliedDPI"))?;

    Ok(())
}
