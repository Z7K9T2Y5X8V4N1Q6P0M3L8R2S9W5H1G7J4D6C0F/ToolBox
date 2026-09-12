//! Context menu construction and handling for the window visual styles ListView.
//!
//! Provides command IDs and helper functions to display the right-click
//! context menu for selected system processes.

use rust_i18n::t;
use winsafe::{AnyResult, HMENU, HWND, MenuItem, POINT, co};

/// Command ID for applying basic visual style to the selected process window.
pub const IDM_VISUAL_STYLES_APPLY_BASIC: u16 = 3101;

/// Command ID for applying classic visual style to the selected process window.
pub const IDM_VISUAL_STYLES_APPLY_CLASSIC: u16 = 3102;

/// Construct and display the context menu at the specified screen coordinates.
///
/// Builds a transient popup menu with options to apply visual styles, tracks
/// user interaction, and destroys the menu handle when dismissed.
pub(super) fn show_process_context_menu(
    parent_window_hwnd: &HWND,
    screen_position: POINT,
) -> AnyResult<()> {
    let mut popup_menu = HMENU::CreatePopupMenu()?;

    popup_menu.append_item(&[
        MenuItem::Entry {
            cmd_id: IDM_VISUAL_STYLES_APPLY_BASIC,
            text: &t!("MENU_VISUAL_STYLES_APPLY_BASIC"),
        },
        MenuItem::Entry {
            cmd_id: IDM_VISUAL_STYLES_APPLY_CLASSIC,
            text: &t!("MENU_VISUAL_STYLES_APPLY_CLASSIC"),
        },
    ])?;

    popup_menu.TrackPopupMenu(
        co::TPM::LEFTALIGN | co::TPM::RIGHTBUTTON,
        screen_position,
        parent_window_hwnd,
    )?;

    popup_menu.DestroyMenu()?;

    Ok(())
}
