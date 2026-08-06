use winsafe::prelude::{GuiEventsParent, GuiWindow};

use super::ids::IDM_OPTIONS_RESTART_EXPLORER;
use crate::{
    MainWindow,
    ui::menu::ids::{IDM_LANG_EN_US, IDM_LANG_ZH_CN},
};

pub fn register_menu_events(main_window_instance: &MainWindow) {
    main_window_instance
        .main_window
        .on()
        .wm_command_acc_menu(IDM_OPTIONS_RESTART_EXPLORER, move || Ok(()));

    let cloned_main_window_instance_for_en = main_window_instance.clone();
    main_window_instance
        .main_window
        .on()
        .wm_command_acc_menu(IDM_LANG_EN_US, move || {
            rust_i18n::set_locale("en-US");

            let main_window_hwnd = cloned_main_window_instance_for_en.main_window.hwnd();
            main_window_hwnd.SetMenu(&super::build_main_menu()?)?;

            Ok(())
        });

    let cloned_main_window_instance_for_en = main_window_instance.clone();
    main_window_instance
        .main_window
        .on()
        .wm_command_acc_menu(IDM_LANG_ZH_CN, move || {
            rust_i18n::set_locale("zh-CN");

            let main_window_hwnd = cloned_main_window_instance_for_en.main_window.hwnd();
            main_window_hwnd.SetMenu(&super::build_main_menu()?)?;

            Ok(())
        });
}
