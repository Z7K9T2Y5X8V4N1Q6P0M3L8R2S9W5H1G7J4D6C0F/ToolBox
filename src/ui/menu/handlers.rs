use winsafe::prelude::{GuiEventsParent, GuiWindow};

use super::ids::IDM_OPTIONS_RESTART_EXPLORER;
use crate::{
    MainWindow,
    config::{AppConfig, AppLanguage},
    ui::menu::ids::{IDM_LANG_EN_US, IDM_LANG_ZH_CN},
};

pub fn register_menu_events(main_window_instance: &MainWindow) {
    main_window_instance
        .main_window
        .on()
        .wm_command_acc_menu(IDM_OPTIONS_RESTART_EXPLORER, move || Ok(()));

    let cloned_main_window_instance_for_en_us = main_window_instance.clone();
    main_window_instance
        .main_window
        .on()
        .wm_command_acc_menu(IDM_LANG_EN_US, move || {
            apply_language_change(
                &cloned_main_window_instance_for_en_us,
                "en-US",
                AppLanguage::EnUs,
            )
        });

    let cloned_main_window_instance_for_zh_cn = main_window_instance.clone();
    main_window_instance
        .main_window
        .on()
        .wm_command_acc_menu(IDM_LANG_ZH_CN, move || {
            apply_language_change(
                &cloned_main_window_instance_for_zh_cn,
                "zh-CN",
                AppLanguage::ZhCn,
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
    main_window_hwnd.SetMenu(&super::build_main_menu()?)?;
    main_window_hwnd.SetWindowText(&rust_i18n::t!("TOOLBOX_TITLE"))?;

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
