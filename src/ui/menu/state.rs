//! Menu command ID constants.
//!
//! Each constant is a unique `u16` sent via `WM_COMMAND` when the user
//! clicks the corresponding menu item. Ranges are grouped by submenu:
//! - `1000–1999` — Options submenu
//! - `2000–2999` — Language submenu

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
