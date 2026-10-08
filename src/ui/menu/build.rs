//! Menu bar construction and lifecycle management.
//!
//! Provides RAII guard primitives for detached Win32 [`HMENU`] instances to ensure
//! complete leak prevention during creation failures, alongside clean menu bar replacement
//! and non-client metric synchronization.

use anyhow::{Context, Result};
use rust_i18n::t;
use windows::Win32::{Foundation::HWND as RawHwnd, UI::WindowsAndMessaging::DrawMenuBar};
use winsafe::{BmpPtrStr, HMENU, HWND, IdMenu, MenuItem, co};

use super::state::{
    IDM_OPTIONS_ADD_EXTRA_CLASSIC_VISUAL_STYLES, IDM_OPTIONS_REPAIR_VISUAL_STYLES_TO_DEFAULT,
    IDM_OPTIONS_RESTART_EXPLORER, IDM_OPTIONS_RESTORE_DEFAULT_CLASSIC_VISUAL_STYLES,
    IDM_OPTIONS_TOGGLE_GLOBAL_BASIC_STYLES, IDM_OPTIONS_TOGGLE_GLOBAL_CLASSIC_STYLES,
    is_global_basic_styles_active,
};
use crate::config::AppLanguage;

/// RAII scope guard holding an unattached [`HMENU`] handle.
///
/// Win32 menus that are not yet associated with a window or embedded inside a parent
/// menu are owned by the process. If an error occurs prior to mounting, this guard
/// invokes [`HMENU::DestroyMenu`] upon drop to prevent GDI/User handle leaks.
///
/// Once ownership has been transferred to a window or a parent menu, the guard is
/// consumed via [`UnattachedMenuGuard::append_submenu`] or
/// [`UnattachedMenuGuard::attach_to_window`].
pub struct UnattachedMenuGuard {
    menu_handle: Option<HMENU>,
}

impl UnattachedMenuGuard {
    /// Encapsulate a newly created unattached menu handle inside an RAII scope guard.
    pub const fn new(menu_handle: HMENU) -> Self {
        Self {
            menu_handle: Some(menu_handle),
        }
    }

    /// Borrow the underlying [`HMENU`] handle.
    pub fn handle(&self) -> Result<&HMENU> {
        self.menu_handle
            .as_ref()
            .context(t!("ERROR_WINDOW_MENU_ALREADY_ATTACHED"))
    }

    /// Append a child popup submenu to this menu, transferring its ownership into the menu tree.
    ///
    /// Consumes the child `submenu_guard` by value. If the Win32 append operation succeeds,
    /// ownership of the submenu handle is permanently assumed by Windows (parent menu),
    /// and the child guard is cleanly disarmed without triggering destruction.
    pub fn append_submenu(&self, text: &str, mut submenu_guard: Self) -> Result<()> {
        let parent_menu_handle = self.handle()?;
        let child_menu_handle = submenu_guard.handle()?;

        parent_menu_handle
            .append_item(&[MenuItem::Submenu {
                submenu: child_menu_handle,
                text,
            }])
            .context(t!("ERROR_WINDOW_APPEND_MENU_FAILED"))?;

        // Mounting succeeded: disarm the child guard so Windows owns the child menu.
        let _ = submenu_guard.menu_handle.take();

        Ok(())
    }

    /// Atomically transfer ownership of this menu to the specified window.
    ///
    /// Replaces the window's existing menu, destroys the detached previous menu (if present),
    /// updates the window's non-client menu metrics via [`DrawMenuBar`], and consumes this guard.
    pub fn attach_to_window(mut self, target_window_hwnd: &HWND) -> Result<()> {
        let previous_menu_handle = target_window_hwnd.GetMenu();

        // 1. Attempt to mount the new menu first. If this operation fails, `self` remains
        // intact and its `Drop` implementation will automatically destroy the unattached menu.
        target_window_hwnd
            .SetMenu(self.handle()?)
            .context(t!("ERROR_WINDOW_SET_MENU_FAILED"))?;

        // 2. Once mounting succeeds, disarm the guard by taking ownership out of the Option.
        let _ = self.menu_handle.take();

        // 3. Destroy the previously mounted menu which is now detached from the window.
        if let Some(mut previous_unattached_menu) = previous_menu_handle {
            previous_unattached_menu
                .DestroyMenu()
                .context(t!("ERROR_WINDOW_DESTROY_MENU_FAILED"))?;
        }

        let target_window_raw_hwnd = RawHwnd(target_window_hwnd.ptr());
        let _ = unsafe { DrawMenuBar(target_window_raw_hwnd) };

        Ok(())
    }
}

impl Drop for UnattachedMenuGuard {
    fn drop(&mut self) {
        if let Some(mut unattached_menu_handle) = self.menu_handle.take() {
            let _ = unattached_menu_handle.DestroyMenu();
        }
    }
}

/// Build the complete main menu bar using the current locale wrapped in an RAII guard.
///
/// Returns an [`UnattachedMenuGuard`] holding the newly created menu tree.
pub fn build_main_menu() -> Result<UnattachedMenuGuard> {
    let root_menu_bar_handle =
        HMENU::CreateMenu().context(t!("ERROR_WINDOW_CREATE_MENU_FAILED"))?;
    let root_menu_bar_guard = UnattachedMenuGuard::new(root_menu_bar_handle);

    let options_popup_menu_guard = create_options_popup_menu()?;
    let language_popup_menu_guard = create_language_popup_menu()?;

    // Ownership of both submenus is transferred by value directly into root_menu_bar_guard.
    root_menu_bar_guard.append_submenu(&t!("MENU_OPTIONS"), options_popup_menu_guard)?;
    root_menu_bar_guard.append_submenu(&t!("MENU_LANGUAGE"), language_popup_menu_guard)?;

    Ok(root_menu_bar_guard)
}

/// Build the Options submenu wrapped in an RAII guard.
fn create_options_popup_menu() -> Result<UnattachedMenuGuard> {
    let options_popup_menu_handle =
        HMENU::CreatePopupMenu().context(t!("ERROR_WINDOW_CREATE_MENU_FAILED"))?;
    let options_popup_menu_guard = UnattachedMenuGuard::new(options_popup_menu_handle);
    let popup_menu_handle = options_popup_menu_guard.handle()?;

    popup_menu_handle
        .append_item(&[
            MenuItem::Entry {
                cmd_id: IDM_OPTIONS_RESTART_EXPLORER,
                text: &t!("MENU_OPTIONS_RESTART_EXPLORER"),
            },
            MenuItem::Separator,
            MenuItem::Entry {
                cmd_id: IDM_OPTIONS_REPAIR_VISUAL_STYLES_TO_DEFAULT,
                text: &t!("MENU_OPTIONS_REPAIR_VISUAL_STYLES_TO_DEFAULT"),
            },
            MenuItem::Separator,
            MenuItem::Entry {
                cmd_id: IDM_OPTIONS_RESTORE_DEFAULT_CLASSIC_VISUAL_STYLES,
                text: &t!("MENU_OPTIONS_RESTORE_DEFAULT_CLASSIC_VISUAL_STYLES"),
            },
            MenuItem::Entry {
                cmd_id: IDM_OPTIONS_ADD_EXTRA_CLASSIC_VISUAL_STYLES,
                text: &t!("MENU_OPTIONS_ADD_EXTRA_CLASSIC_VISUAL_STYLES"),
            },
            MenuItem::Separator,
        ])
        .context(t!("ERROR_WINDOW_APPEND_MENU_FAILED"))?;

    let is_basic_styles_active = is_global_basic_styles_active();
    let basic_styles_menu_flags = if is_basic_styles_active {
        co::MF::STRING | co::MF::CHECKED
    } else {
        co::MF::STRING | co::MF::UNCHECKED
    };

    popup_menu_handle
        .AppendMenu(
            basic_styles_menu_flags,
            IdMenu::Id(IDM_OPTIONS_TOGGLE_GLOBAL_BASIC_STYLES),
            BmpPtrStr::from_str(&t!("MENU_OPTIONS_TOGGLE_GLOBAL_BASIC_STYLES")),
        )
        .context(t!("ERROR_WINDOW_APPEND_MENU_FAILED"))?;

    popup_menu_handle
        .append_item(&[MenuItem::Entry {
            cmd_id: IDM_OPTIONS_TOGGLE_GLOBAL_CLASSIC_STYLES,
            text: &t!("MENU_OPTIONS_TOGGLE_GLOBAL_CLASSIC_STYLES"),
        }])
        .context(t!("ERROR_WINDOW_APPEND_MENU_FAILED"))?;

    Ok(options_popup_menu_guard)
}

/// Determine the menu item state flags based on whether the item matches the active language.
fn resolve_language_menu_item_flags(is_active_language: bool) -> co::MF {
    if is_active_language {
        co::MF::STRING | co::MF::CHECKED | co::MF::GRAYED
    } else {
        co::MF::STRING
    }
}

/// Build the Language submenu wrapped in an RAII guard.
fn create_language_popup_menu() -> Result<UnattachedMenuGuard> {
    let language_popup_menu_handle =
        HMENU::CreatePopupMenu().context(t!("ERROR_WINDOW_CREATE_MENU_FAILED"))?;
    let language_popup_menu_guard = UnattachedMenuGuard::new(language_popup_menu_handle);
    let popup_menu_handle = language_popup_menu_guard.handle()?;

    let current_locale_tag = rust_i18n::locale();
    let current_active_language =
        AppLanguage::from_locale_tag(&current_locale_tag).unwrap_or(AppLanguage::EnUs);

    for supported_language in AppLanguage::all() {
        let is_active_language = *supported_language == current_active_language;
        let menu_item_flags = resolve_language_menu_item_flags(is_active_language);

        popup_menu_handle
            .AppendMenu(
                menu_item_flags,
                IdMenu::Id(supported_language.menu_command_id()),
                BmpPtrStr::from_str(supported_language.native_display_name()),
            )
            .context(t!("ERROR_WINDOW_APPEND_MENU_FAILED"))?;
    }

    Ok(language_popup_menu_guard)
}

/// Destroy the current menu bar and replace it with a freshly built one.
///
/// Builds a new menu tree protected by an RAII guard, atomically attaches it to
/// the target window while reclaiming previous menu handles, and triggers non-client redraw.
pub(super) fn rebuild_main_menu(main_window_hwnd: &HWND) -> Result<()> {
    let new_menu_bar_guard = build_main_menu()?;
    new_menu_bar_guard.attach_to_window(main_window_hwnd)?;
    Ok(())
}
