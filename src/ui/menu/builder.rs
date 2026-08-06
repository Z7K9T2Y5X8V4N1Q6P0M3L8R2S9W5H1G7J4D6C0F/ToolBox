use winsafe::{HMENU, MenuItem};

use super::ids::IDM_OPTIONS_RESTART_EXPLORER;
use crate::ui::menu::ids::{IDM_LANG_EN_US, IDM_LANG_ZH_CN};

pub fn build_main_menu() -> winsafe::AnyResult<HMENU> {
    let main_menu_bar = HMENU::CreateMenu()?;

    let options_popup_menu = HMENU::CreatePopupMenu()?;
    options_popup_menu.append_item(&[MenuItem::Entry {
        cmd_id: IDM_OPTIONS_RESTART_EXPLORER,
        text: "重启文件资源管理器",
    }])?;

    main_menu_bar.append_item(&[MenuItem::Submenu {
        submenu: &options_popup_menu,
        text: "选项",
    }])?;

    let language_popup_menu = HMENU::CreatePopupMenu()?;
    language_popup_menu.append_item(&[
        MenuItem::Entry {
            cmd_id: IDM_LANG_EN_US,
            text: "English",
        },
        MenuItem::Entry {
            cmd_id: IDM_LANG_ZH_CN,
            text: "简体中文",
        },
    ])?;

    main_menu_bar.append_item(&[MenuItem::Submenu {
        submenu: &language_popup_menu,
        text: "语言",
    }])?;

    Ok(main_menu_bar)
}
