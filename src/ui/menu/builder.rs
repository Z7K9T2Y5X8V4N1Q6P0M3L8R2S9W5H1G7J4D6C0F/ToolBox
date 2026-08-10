use rust_i18n::t;
use winsafe::{BmpPtrStr, HMENU, IdMenu, MenuItem, co};

use super::ids::IDM_OPTIONS_RESTART_EXPLORER;
use crate::ui::menu::ids::{IDM_LANG_EN_US, IDM_LANG_ZH_CN};

pub fn build_main_menu() -> winsafe::AnyResult<HMENU> {
    let main_menu_bar = HMENU::CreateMenu()?;
    let options_popup_menu = HMENU::CreatePopupMenu()?;

    options_popup_menu.append_item(&[MenuItem::Entry {
        cmd_id: IDM_OPTIONS_RESTART_EXPLORER,
        text: &t!("MENU_OPTIONS_RESTART_EXPLORER"),
    }])?;
    let language_popup_menu = HMENU::CreatePopupMenu()?;
    append_language_menu_items(&language_popup_menu)?;

    main_menu_bar.append_item(&[MenuItem::Submenu {
        submenu: &options_popup_menu,
        text: &t!("MENU_OPTIONS"),
    }])?;
    main_menu_bar.append_item(&[MenuItem::Submenu {
        submenu: &language_popup_menu,
        text: &t!("MENU_LANGUAGE"),
    }])?;

    Ok(main_menu_bar)
}

fn append_language_menu_items(language_popup_menu: &HMENU) -> winsafe::AnyResult<()> {
    let current_locale = rust_i18n::locale();
    let is_current_locale = |locale: &str| &*current_locale == locale;

    append_language_entry(
        language_popup_menu,
        IDM_LANG_EN_US,
        "English",
        is_current_locale("en-US"),
    )?;
    append_language_entry(
        language_popup_menu,
        IDM_LANG_ZH_CN,
        "简体中文",
        is_current_locale("zh-CN"),
    )?;

    Ok(())
}

fn append_language_entry(
    language_popup_menu: &HMENU,
    cmd_id: u16,
    display_text: &str,
    is_active_locale: bool,
) -> winsafe::AnyResult<()> {
    let flags = if is_active_locale {
        co::MF::STRING | co::MF::CHECKED | co::MF::GRAYED
    } else {
        co::MF::STRING
    };
    language_popup_menu.AppendMenu(flags, IdMenu::Id(cmd_id), BmpPtrStr::from_str(display_text))?;

    Ok(())
}
