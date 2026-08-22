use winsafe::prelude::{GuiEventsParent, GuiWindow};

use super::ids::IDM_OPTIONS_RESTART_EXPLORER;
use crate::{
    config::{AppConfig, AppLanguage},
    core::MainWindow,
    ui::menu::ids::{IDM_LANG_EN_US, IDM_LANG_ZH_CN},
};

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

fn register_language_menu_handler(
    main_window_instance: &MainWindow,
    menu_command_id: u16,
    locale_string: &'static str,
    target_language: AppLanguage,
) {
    let cloned_main_window_instance_language_menu_handler = main_window_instance.clone();
    main_window_instance
        .main_window
        .on()
        .wm_command_acc_menu(menu_command_id, move || {
            apply_language_change(
                &cloned_main_window_instance_language_menu_handler,
                locale_string,
                target_language,
            )
        });
}

fn apply_language_change(
    main_window_instance: &MainWindow,
    locale: &str,
    language: AppLanguage,
) -> winsafe::AnyResult<()> {
    rust_i18n::set_locale(locale);

    let main_window_hwnd = main_window_instance.main_window.hwnd();
    rebuild_main_menu(&main_window_hwnd)?;
    main_window_hwnd.SetWindowText(&rust_i18n::t!("TOOLBOX_TITLE"))?;

    main_window_instance.tab_pages.update_tab_control_titles()?;
    main_window_instance.tab_pages.update_page_contents()?;

    let mut config = AppConfig::load();
    config.language = language;

    if let Err(save_error) = config.save() {
        let error_message =
            rust_i18n::t!("CONFIG_SAVE_FAILED", save_error = save_error).to_string();
        *main_window_instance.pending_error_message.borrow_mut() = Some(error_message);

        unsafe {
            main_window_hwnd.PostMessage(winsafe::msg::Wm {
                msg_id: winsafe::co::WM::APP,
                wparam: 0,
                lparam: 0,
            })?;
        }
    }

    Ok(())
}

fn rebuild_main_menu(main_window_hwnd: &winsafe::HWND) -> winsafe::AnyResult<()> {
    let old_hmenu = main_window_hwnd.GetMenu();
    main_window_hwnd.SetMenu(&super::build_main_menu()?)?;
    if let Some(mut old_hmenu) = old_hmenu {
        old_hmenu.DestroyMenu()?;
    }
    Ok(())
}
