use std::ops::Deref;

use rust_i18n::t;
use winsafe::{BmpPtrStr, HMENU, IdMenu, MenuItem, co};

use super::ids::IDM_OPTIONS_RESTART_EXPLORER;
use crate::ui::menu::ids::{IDM_LANG_EN_US, IDM_LANG_ZH_CN};

pub fn build_main_menu() -> winsafe::AnyResult<HMENU> {
    let main_menu_bar = HMENU::CreateMenu()?;

    let options_popup_menu = create_options_popup_menu()?;
    let language_popup_menu = create_language_popup_menu()?;

    main_menu_bar.append_item(&[
        MenuItem::Submenu {
            submenu: &options_popup_menu,
            text: &t!("MENU_OPTIONS"),
        },
        MenuItem::Submenu {
            submenu: &language_popup_menu,
            text: &t!("MENU_LANGUAGE"),
        },
    ])?;

    Ok(main_menu_bar)
}

fn create_options_popup_menu() -> winsafe::AnyResult<HMENU> {
    let options_popup_menu = HMENU::CreatePopupMenu()?;
    options_popup_menu.append_item(&[MenuItem::Entry {
        cmd_id: IDM_OPTIONS_RESTART_EXPLORER,
        text: &t!("MENU_OPTIONS_RESTART_EXPLORER"),
    }])?;
    Ok(options_popup_menu)
}

fn create_language_popup_menu() -> winsafe::AnyResult<HMENU> {
    let language_popup_menu = HMENU::CreatePopupMenu()?;
    let current_locale = rust_i18n::locale();

    let languages = [
        (IDM_LANG_EN_US, "English", "en-US"),
        (IDM_LANG_ZH_CN, "简体中文", "zh-CN"),
    ];

    for (menu_command_id, display_text, locale) in languages {
        let is_active_locale = current_locale.deref() == locale;
        let menu_item_flags = if is_active_locale {
            co::MF::STRING | co::MF::CHECKED | co::MF::GRAYED
        } else {
            co::MF::STRING
        };
        language_popup_menu.AppendMenu(
            menu_item_flags,
            IdMenu::Id(menu_command_id),
            BmpPtrStr::from_str(display_text),
        )?;
    }

    Ok(language_popup_menu)
}
