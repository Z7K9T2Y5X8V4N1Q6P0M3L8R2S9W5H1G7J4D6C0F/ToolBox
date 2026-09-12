//! Menu event handlers.
//!
//! Registers WM_COMMAND handlers for all menu items on [`MainWindow`].
//! Language items rebuild the entire UI text and persist the new preference.

use rust_i18n::t;
use winsafe::{
    AnyResult, co, msg,
    prelude::{GuiEventsParent, GuiWindow},
};

use crate::{
    app::MainWindow,
    config::{AppConfig, AppLanguage},
};

use super::state::{IDM_LANG_EN_US, IDM_LANG_ZH_CN, IDM_OPTIONS_RESTART_EXPLORER};

/// Register WM_COMMAND handlers for all menu items.
///
/// Must be called once during main window event setup, before the
/// message loop starts.
pub fn register_menu_events(main_window_instance: &MainWindow) {
    main_window_instance
        .main_window
        .on()
        .wm_command_acc_menu(IDM_OPTIONS_RESTART_EXPLORER, move || Ok(()));

    for (menu_command_id, locale_string, target_language) in [
        (IDM_LANG_EN_US, "en-US", AppLanguage::EnUs),
        (IDM_LANG_ZH_CN, "zh-CN", AppLanguage::ZhCn),
    ] {
        register_language_menu_handler(
            main_window_instance,
            menu_command_id,
            locale_string,
            target_language,
        );
    }
}

/// Register a WM_COMMAND handler for a single language menu item.
fn register_language_menu_handler(
    main_window_instance: &MainWindow,
    menu_command_id: u16,
    locale_string: &'static str,
    target_language: AppLanguage,
) {
    let cloned_main_window_instance = main_window_instance.clone();
    main_window_instance
        .main_window
        .on()
        .wm_command_acc_menu(menu_command_id, move || {
            apply_language_change(&cloned_main_window_instance, locale_string, target_language)
        });
}

/// Switch the application locale, rebuild all UI text, and persist the choice.
///
/// # Steps
/// 1. Update the `rust_i18n` locale so all subsequent `t!()` calls use the new language.
/// 2. Rebuild the menu bar so its labels are re-translated.
/// 3. Update the window title and all tab/page text labels.
/// 4. Save the new language preference to the config file.
///
/// # Error deferral
/// If saving the config fails, the error is stored in
/// [`MainWindow::pending_error_message`] and a [`winsafe::co::WM::APP`] message
/// is posted rather than showing a dialog immediately. This avoids reentrancy
/// issues that can occur when a modal dialog is opened inside a menu handler.
fn apply_language_change(
    main_window_instance: &MainWindow,
    locale: &str,
    language: AppLanguage,
) -> AnyResult<()> {
    rust_i18n::set_locale(locale);

    let main_window_hwnd = main_window_instance.main_window.hwnd();
    super::build::rebuild_main_menu(&main_window_hwnd)?;
    main_window_hwnd.SetWindowText(&t!("TOOLBOX_TITLE"))?;

    main_window_instance
        .tab_container
        .update_tab_control_titles()?;
    main_window_instance.tab_container.update_page_contents()?;

    let mut config = AppConfig::load();
    config.language = language;

    if let Err(save_error) = config.save() {
        let error_message = t!("CONFIG_SAVE_FAILED", save_error = save_error).to_string();
        *main_window_instance.pending_error_message.borrow_mut() = Some(error_message);

        unsafe {
            main_window_hwnd.PostMessage(msg::Wm {
                msg_id: co::WM::APP,
                wparam: 0,
                lparam: 0,
            })?;
        }
    }

    Ok(())
}
