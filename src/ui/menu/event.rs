//! Menu event handlers.
//!
//! Registers WM_COMMAND handlers for all menu items on [`MainWindow`].
//! Language items rebuild the entire UI text and persist the new preference.

use rust_i18n::t;
use winsafe::{
    AnyResult,
    prelude::{GuiEventsParent, GuiWindow},
};

use crate::{
    app::{MainWindow, UserConfirmationOutcome},
    config::{AppConfig, AppLanguage},
    desktop,
};

use super::state::{
    IDM_LANG_EN_US, IDM_LANG_ZH_CN, IDM_OPTIONS_ADD_EXTRA_CLASSIC_VISUAL_STYLES,
    IDM_OPTIONS_REPAIR_VISUAL_STYLES_TO_DEFAULT, IDM_OPTIONS_RESTART_EXPLORER,
    IDM_OPTIONS_RESTORE_DEFAULT_CLASSIC_VISUAL_STYLES, IDM_OPTIONS_TOGGLE_GLOBAL_BASIC_STYLES,
    IDM_OPTIONS_TOGGLE_GLOBAL_CLASSIC_STYLES,
};

/// Register WM_COMMAND handlers for all menu items.
///
/// Must be called once during main window event setup, before the
/// message loop starts.
pub fn register_menu_events(main_window_instance: &MainWindow) {
    let cloned_main_window_for_restart_explorer = main_window_instance.clone();
    main_window_instance.main_window.on().wm_command_acc_menu(
        IDM_OPTIONS_RESTART_EXPLORER,
        move || {
            let confirmation = cloned_main_window_for_restart_explorer.prompt_confirmation(
                &t!("MENU_OPTIONS_RESTART_EXPLORER"),
                &t!("CONFIRM_RESTART_EXPLORER_MESSAGE"),
            )?;

            if confirmation != UserConfirmationOutcome::Confirmed {
                return Ok(());
            }

            if let Err(restart_error) = desktop::restart_desktop_shell() {
                cloned_main_window_for_restart_explorer
                    .post_deferred_error(restart_error.to_string());
            }

            Ok(())
        },
    );

    let cloned_main_window_for_repair_styles = main_window_instance.clone();
    main_window_instance.main_window.on().wm_command_acc_menu(
        IDM_OPTIONS_REPAIR_VISUAL_STYLES_TO_DEFAULT,
        move || {
            let confirmation = cloned_main_window_for_repair_styles.prompt_confirmation(
                &t!("MENU_OPTIONS_REPAIR_VISUAL_STYLES_TO_DEFAULT"),
                &t!("CONFIRM_REPAIR_VISUAL_STYLES_MESSAGE"),
            )?;

            if confirmation != UserConfirmationOutcome::Confirmed {
                return Ok(());
            }

            match desktop::theme::apply_default_metrics() {
                Ok(()) => {
                    cloned_main_window_for_repair_styles.show_info_dialog(
                        &t!("MENU_OPTIONS_REPAIR_VISUAL_STYLES_TO_DEFAULT"),
                        &t!("REPAIR_VISUAL_STYLES_SUCCESS"),
                    )?;
                }
                Err(repair_error) => {
                    cloned_main_window_for_repair_styles
                        .post_deferred_error(repair_error.to_string());
                }
            }

            Ok(())
        },
    );

    main_window_instance.main_window.on().wm_command_acc_menu(
        IDM_OPTIONS_RESTORE_DEFAULT_CLASSIC_VISUAL_STYLES,
        move || Ok(()),
    );

    main_window_instance
        .main_window
        .on()
        .wm_command_acc_menu(IDM_OPTIONS_ADD_EXTRA_CLASSIC_VISUAL_STYLES, move || Ok(()));

    main_window_instance
        .main_window
        .on()
        .wm_command_acc_menu(IDM_OPTIONS_TOGGLE_GLOBAL_BASIC_STYLES, move || Ok(()));

    main_window_instance
        .main_window
        .on()
        .wm_command_acc_menu(IDM_OPTIONS_TOGGLE_GLOBAL_CLASSIC_STYLES, move || Ok(()));

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
/// If saving the config fails, the error is queued via
/// [`MainWindow::post_deferred_error`] rather than showing a dialog immediately.
/// This avoids reentrancy issues that can occur when a modal dialog is opened inside a menu handler.
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
        main_window_instance.post_deferred_error(error_message);
    }

    Ok(())
}
