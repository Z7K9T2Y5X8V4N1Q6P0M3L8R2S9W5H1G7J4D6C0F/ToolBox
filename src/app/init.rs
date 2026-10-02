//! Application initialization routines that run before window creation and pre-flight checks.
//!
//! Call order matters here:
//! 1. Logger must be initialized first to capture all subsequent log output.
//! 2. System-level locale is applied as an early safety net so any failure dialogs
//!    during configuration loading or hook setup are presented in the system language.
//! 3. UI customization hook must be installed before any window creation.
//! 4. Panic hook replaces default stderr output with a localized GUI dialog.
//! 5. Config locale overrides early system locale with user-persisted preference.

use crate::{
    config::{AppConfig, AppLanguage},
    error,
    ui::window,
};

/// Run all initialization steps in the correct order.
///
/// This must be called once, at the very beginning of [`crate::main`],
/// before checking elevation, acquiring single-instance locks, or creating Win32 windows.
///
/// # Initialization sequence
/// 1. Logger — captures debug output from all subsequent steps
/// 2. Early system locale — ensures configuration error dialogs match system language
/// 3. UI customization hook — must be installed before any window creation
/// 4. Panic hook — replaces default stderr output with a GUI error dialog
/// 5. Persisted config locale — overrides early locale with user saved preference
pub fn initialize_application() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    setup_early_system_locale();
    window::install_ui_customization_hook();
    error::install_panic_hook();
    setup_persisted_config_locale();
}

/// Configure the initial locale based strictly on operating system detection.
///
/// Acts as an early safety net before [`AppConfig::load`] executes. If the config
/// file is corrupt or unreadable, the resulting error dialog will already be rendered
/// in the user's system language rather than falling back to the hardcoded compile-time default.
fn setup_early_system_locale() {
    let system_language = AppLanguage::detect_from_system();
    rust_i18n::set_locale(system_language.as_locale_str());
}

/// Override the early locale with the user's saved preference from the configuration file.
///
/// On the first run, [`AppConfig::load`] generates a default configuration derived from
/// the system language, remaining consistent with [`setup_early_system_locale`]. On subsequent
/// runs, user-saved preferences take precedence.
fn setup_persisted_config_locale() {
    let app_config = AppConfig::load();
    rust_i18n::set_locale(app_config.language.as_locale_str());
}
