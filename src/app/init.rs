//! Application initialization routines that run before the main window is created.
//!
//! Call order matters here: the logger must be initialized first to capture all
//! subsequent log output, locale must be set before any UI text is rendered,
//! and the UI hook must be installed before any window is created.

use crate::{
    config::{AppConfig, AppLanguage},
    error,
    ui::window,
};

/// Run all initialization steps in the correct order.
///
/// This must be called once, at the very start of [`MainWindow::create_and_run`],
/// before any Win32 window or control is created.
///
/// # Initialization sequence
/// 1. Logger — captures debug output from all subsequent steps
/// 2. Initial locale — ensures early UI elements use the system language
/// 3. UI customization hook — must be installed before any window creation
/// 4. Panic hook — replaces default stderr output with a GUI error dialog
/// 5. Config locale — overrides system locale with user preference if available
pub fn initialize_application() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    setup_initial_locale();
    window::install_ui_customization_hook();
    error::install_panic_hook();
    setup_config_locale();
}

/// Set the locale to the system default before config is loaded.
///
/// This ensures early UI elements (e.g. config error dialogs) are localized
/// using the system language rather than the hardcoded fallback.
fn setup_initial_locale() {
    let system_locale =
        sys_locale::get_locale().unwrap_or_else(|| AppLanguage::EnUs.as_locale_str().to_string());
    rust_i18n::set_locale(&system_locale);
}

/// Override the locale with the user's saved preference from config.
///
/// Called after [`setup_initial_locale`] so that if the config is missing
/// or corrupt, the error dialog is still shown in the system language.
fn setup_config_locale() {
    let app_config = AppConfig::load();
    rust_i18n::set_locale(&app_config.language.as_locale_str());
}
