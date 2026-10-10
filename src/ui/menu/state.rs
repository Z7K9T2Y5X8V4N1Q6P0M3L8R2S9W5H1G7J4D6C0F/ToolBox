//! Menu command ID constants and toggle states.
//!
//! Each constant is a unique `u16` sent via `WM_COMMAND` when the user
//! clicks the corresponding menu item. Ranges are grouped by submenu:
//! - `1000–1999` — Options submenu
//! - `2000–2999` — Language submenu

use crate::desktop::theme::{
    GlobalBasicStylesState, GlobalBasicStylesWatcher, GlobalClassicStylesState,
    GlobalClassicStylesWatcher,
};
use anyhow::{Context, Result};
use std::sync::Mutex;

/// Trigger an Explorer process restart.
pub const IDM_OPTIONS_RESTART_EXPLORER: u16 = 1001;

/// Repair visual styles back to default.
pub const IDM_OPTIONS_REPAIR_VISUAL_STYLES_TO_DEFAULT: u16 = 1002;

/// Restore default classic visual styles.
pub const IDM_OPTIONS_RESTORE_DEFAULT_CLASSIC_VISUAL_STYLES: u16 = 1003;

/// Add extra classic visual styles.
pub const IDM_OPTIONS_ADD_EXTRA_CLASSIC_VISUAL_STYLES: u16 = 1004;

/// Toggle global basic visual styles.
pub const IDM_OPTIONS_TOGGLE_GLOBAL_BASIC_STYLES: u16 = 1005;

/// Toggle global classic visual styles.
pub const IDM_OPTIONS_TOGGLE_GLOBAL_CLASSIC_STYLES: u16 = 1006;

/// Switch the application language to English (United States).
pub const IDM_LANG_EN_US: u16 = 2001;

/// Switch the application language to Simplified Chinese.
pub const IDM_LANG_ZH_CN: u16 = 2002;

/// Global singleton instance of the basic styles watcher.
static GLOBAL_BASIC_STYLES_WATCHER: Mutex<Option<GlobalBasicStylesWatcher>> = Mutex::new(None);

/// Query whether the global basic visual style watcher is currently enabled.
pub fn is_global_basic_styles_active() -> bool {
    GLOBAL_BASIC_STYLES_WATCHER
        .lock()
        .map_or(false, |watcher_mutex_guard| {
            watcher_mutex_guard
                .as_ref()
                .map_or(false, |basic_styles_watcher| {
                    basic_styles_watcher.current_state() == GlobalBasicStylesState::Enabled
                })
        })
}

/// Query whether the global classic visual style is currently active directly from the kernel DACL.
pub fn is_global_classic_styles_active() -> bool {
    GlobalClassicStylesWatcher::new().current_state() == GlobalClassicStylesState::Enabled
}

/// Toggle the global basic visual styles watcher and return the new state.
pub fn toggle_global_basic_styles() -> Result<GlobalBasicStylesState> {
    let mut watcher_mutex_guard = GLOBAL_BASIC_STYLES_WATCHER
        .lock()
        .map_err(|mutex_lock_error| anyhow::anyhow!("{mutex_lock_error}"))
        .context("Failed to lock global basic styles watcher mutex")?;

    let basic_styles_watcher =
        watcher_mutex_guard.get_or_insert_with(GlobalBasicStylesWatcher::new);
    basic_styles_watcher.toggle()
}

/// Toggle the global classic visual styles mode based on live kernel security state.
pub fn toggle_global_classic_styles() -> Result<GlobalClassicStylesState> {
    GlobalClassicStylesWatcher::new().toggle()
}

/// Explicitly stop and dismantle the global basic styles watcher and restore all windows.
///
/// Called during window destruction to guarantee that DWM frame decorations
/// are completely restored before exit, even if the mutex has been poisoned.
pub fn cleanup_global_basic_styles() {
    let mut watcher_mutex_guard = GLOBAL_BASIC_STYLES_WATCHER
        .lock()
        .unwrap_or_else(|watcher_mutex_poison_error| watcher_mutex_poison_error.into_inner());

    if let Some(mut basic_styles_watcher) = watcher_mutex_guard.take() {
        let _ = basic_styles_watcher.stop();
    }
}

/// Explicitly restore standard ThemeSection permissions and disable classic mode.
pub fn cleanup_global_classic_styles() {
    let _ = GlobalClassicStylesWatcher::new().disable();
}
