use crate::config::{AppConfig, AppLanguage};
use crate::{error, ui};

pub fn initialize_application() {
    setup_initial_locale();
    ui::window::ui_customization_hook::install_ui_customization_hook();
    error::panic_hook::install_panic_hook();
    setup_config_locale();
}

fn setup_initial_locale() {
    let system_locale =
        sys_locale::get_locale().unwrap_or_else(|| AppLanguage::EnUs.as_locale_str().to_string());
    rust_i18n::set_locale(&system_locale);
}

fn setup_config_locale() {
    let app_config = AppConfig::load();
    rust_i18n::set_locale(&app_config.language.as_locale_str());
}
