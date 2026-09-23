//! Generic registry import engine for desktop visual styles and metrics.

use anyhow::{Context, Result};

use super::presets::metrics;
use crate::desktop::session;

/// Restore default visual styles, system colors, and non-client metrics for the active user.
///
/// Writes the default presets into the interactive user's registry hive.
/// Note: Windows requires a user sign-out or system restart to reload these metrics.
pub fn apply_default_metrics() -> Result<()> {
    let user_root_key = session::open_active_user_registry_root()?;

    // 1. Write Control Panel\Appearance
    let (appearance_key, _) = user_root_key
        .create_subkey(r"Control Panel\Appearance")
        .context(r"Failed to open/create Control Panel\Appearance")?;
    for (value_name, value_data) in metrics::APPEARANCE_ENTRIES {
        appearance_key.set_value(value_name, &value_data)?;
    }

    // 2. Write Control Panel\Colors
    let (colors_key, _) = user_root_key
        .create_subkey(r"Control Panel\Colors")
        .context(r"Failed to open/create Control Panel\Colors")?;
    for (value_name, value_data) in metrics::COLOR_ENTRIES {
        colors_key.set_value(value_name, &value_data)?;
    }

    // 3. Write Control Panel\Desktop\WindowMetrics
    let (window_metrics_key, _) = user_root_key
        .create_subkey(r"Control Panel\Desktop\WindowMetrics")
        .context(r"Failed to open/create Control Panel\Desktop\WindowMetrics")?;
    for (value_name, value_data) in metrics::WINDOW_METRICS_STRING_ENTRIES {
        window_metrics_key.set_value(value_name, &value_data)?;
    }

    let font_reg_value = metrics::default_font_reg_value();
    for font_key in metrics::WINDOW_METRICS_BINARY_FONT_KEYS {
        window_metrics_key.set_raw_value(font_key, &font_reg_value)?;
    }
    window_metrics_key.set_value("AppliedDPI", &metrics::APPLIED_DPI_VALUE)?;

    Ok(())
}
