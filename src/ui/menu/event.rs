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
    app::MainWindow,
    config::{AppConfig, AppLanguage},
    desktop,
    ui::dialog::{self, UserConfirmationOutcome},
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
            let main_window_hwnd = cloned_main_window_for_restart_explorer.main_window.hwnd();
            let confirmation = dialog::prompt_confirmation_dialog(
                Some(main_window_hwnd),
                &t!("MENU_OPTIONS_RESTART_EXPLORER"),
                &t!("CONFIRM_RESTART_EXPLORER_MESSAGE"),
            )?;

            if confirmation != UserConfirmationOutcome::Confirmed {
                return Ok(());
            }

            if let Err(restart_shell_error) = desktop::shell::restart_desktop_shell() {
                let restart_shell_error_message = restart_shell_error.to_string();
                cloned_main_window_for_restart_explorer
                    .post_deferred_error(restart_shell_error_message);
            }

            Ok(())
        },
    );

    let cloned_main_window_for_repair_styles = main_window_instance.clone();
    main_window_instance.main_window.on().wm_command_acc_menu(
        IDM_OPTIONS_REPAIR_VISUAL_STYLES_TO_DEFAULT,
        move || {
            let main_window_hwnd = cloned_main_window_for_repair_styles.main_window.hwnd();
            let confirmation = dialog::prompt_confirmation_dialog(
                Some(main_window_hwnd),
                &t!("MENU_OPTIONS_REPAIR_VISUAL_STYLES_TO_DEFAULT"),
                &t!("CONFIRM_REPAIR_VISUAL_STYLES_MESSAGE"),
            )?;

            if confirmation != UserConfirmationOutcome::Confirmed {
                return Ok(());
            }

            match desktop::theme::apply_default_metrics() {
                Ok(()) => {
                    dialog::show_info_dialog(
                        Some(main_window_hwnd),
                        &t!("MENU_OPTIONS_REPAIR_VISUAL_STYLES_TO_DEFAULT"),
                        &t!("REPAIR_VISUAL_STYLES_SUCCESS"),
                    )?;
                }
                Err(repair_metrics_error) => {
                    let repair_metrics_error_message = repair_metrics_error.to_string();
                    cloned_main_window_for_repair_styles
                        .post_deferred_error(repair_metrics_error_message);
                }
            }

            Ok(())
        },
    );

    let cloned_main_window_for_restore_classic_schemes = main_window_instance.clone();
    main_window_instance.main_window.on().wm_command_acc_menu(
        IDM_OPTIONS_RESTORE_DEFAULT_CLASSIC_VISUAL_STYLES,
        move || {
            let main_window_hwnd = cloned_main_window_for_restore_classic_schemes
                .main_window
                .hwnd();
            let confirmation = dialog::prompt_confirmation_dialog(
                Some(main_window_hwnd),
                &t!("MENU_OPTIONS_RESTORE_DEFAULT_CLASSIC_VISUAL_STYLES"),
                &t!("CONFIRM_RESTORE_DEFAULT_CLASSIC_VISUAL_STYLES_MESSAGE"),
            )?;

            if confirmation != UserConfirmationOutcome::Confirmed {
                return Ok(());
            }

            match desktop::theme::restore_default_classic_schemes() {
                Ok(()) => {
                    dialog::show_info_dialog(
                        Some(main_window_hwnd),
                        &t!("MENU_OPTIONS_RESTORE_DEFAULT_CLASSIC_VISUAL_STYLES"),
                        &t!("RESTORE_DEFAULT_CLASSIC_VISUAL_STYLES_SUCCESS"),
                    )?;
                }
                Err(restore_schemes_error) => {
                    let restore_schemes_error_message = restore_schemes_error.to_string();
                    cloned_main_window_for_restore_classic_schemes
                        .post_deferred_error(restore_schemes_error_message);
                }
            }

            Ok(())
        },
    );

    let cloned_main_window_for_add_extra_schemes = main_window_instance.clone();
    main_window_instance.main_window.on().wm_command_acc_menu(
        IDM_OPTIONS_ADD_EXTRA_CLASSIC_VISUAL_STYLES,
        move || {
            let main_window_hwnd = cloned_main_window_for_add_extra_schemes.main_window.hwnd();
            let confirmation = dialog::prompt_confirmation_dialog(
                Some(main_window_hwnd),
                &t!("MENU_OPTIONS_ADD_EXTRA_CLASSIC_VISUAL_STYLES"),
                &t!("CONFIRM_ADD_EXTRA_CLASSIC_VISUAL_STYLES_MESSAGE"),
            )?;

            if confirmation != UserConfirmationOutcome::Confirmed {
                return Ok(());
            }

            match desktop::theme::add_extra_classic_schemes() {
                Ok(()) => {
                    dialog::show_info_dialog(
                        Some(main_window_hwnd),
                        &t!("MENU_OPTIONS_ADD_EXTRA_CLASSIC_VISUAL_STYLES"),
                        &t!("ADD_EXTRA_CLASSIC_VISUAL_STYLES_SUCCESS"),
                    )?;
                }
                Err(add_extra_schemes_error) => {
                    let add_extra_schemes_error_message = add_extra_schemes_error.to_string();
                    cloned_main_window_for_add_extra_schemes
                        .post_deferred_error(add_extra_schemes_error_message);
                }
            }

            Ok(())
        },
    );

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

    let mut persisted_app_config = AppConfig::load();
    persisted_app_config.language = language;

    if let Err(config_save_error) = persisted_app_config.save() {
        let config_save_error_message = config_save_error.to_string();
        let deferred_error_message = t!(
            "CONFIG_SAVE_FAILED",
            error_message = config_save_error_message
        )
        .to_string();
        main_window_instance.post_deferred_error(deferred_error_message);
    }

    Ok(())
}
