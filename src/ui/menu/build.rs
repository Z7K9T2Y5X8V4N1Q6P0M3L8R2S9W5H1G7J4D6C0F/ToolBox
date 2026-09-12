//! Menu bar construction.
//!
//! [`build_main_menu`] builds the full menu tree from scratch using the
//! current locale. It is called once on window creation and again after
//! every language change via [`rebuild_main_menu`].

use std::ops::Deref;

use rust_i18n::t;
use winsafe::{AnyResult, BmpPtrStr, HMENU, HWND, IdMenu, MenuItem, co};

use super::state::{IDM_LANG_EN_US, IDM_LANG_ZH_CN, IDM_OPTIONS_RESTART_EXPLORER};

/// Build the complete main menu bar using the current locale.
///
/// Returns a new [`HMENU`] that can be attached to the main window
/// via [`winsafe::HWND::SetMenu`].
pub fn build_main_menu() -> AnyResult<HMENU> {
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

/// Build the Options submenu.
fn create_options_popup_menu() -> AnyResult<HMENU> {
    let options_popup_menu = HMENU::CreatePopupMenu()?;
    options_popup_menu.append_item(&[MenuItem::Entry {
        cmd_id: IDM_OPTIONS_RESTART_EXPLORER,
        text: &t!("MENU_OPTIONS_RESTART_EXPLORER"),
    }])?;
    Ok(options_popup_menu)
}

/// Build the Language submenu.
///
/// The currently active locale is shown as checked and grayed so the user
/// can see the selection but cannot re-select it.
fn create_language_popup_menu() -> AnyResult<HMENU> {
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

/// Destroy the current menu bar and replace it with a freshly built one.
///
/// Called after a language change so all menu labels reflect the new locale.
/// The old [`HMENU`] is explicitly destroyed to avoid a resource leak.
pub(super) fn rebuild_main_menu(main_window_hwnd: &HWND) -> AnyResult<()> {
    let old_hmenu = main_window_hwnd.GetMenu();
    main_window_hwnd.SetMenu(&build_main_menu()?)?;
    if let Some(mut old_hmenu) = old_hmenu {
        old_hmenu.DestroyMenu()?;
    }
    Ok(())
}
