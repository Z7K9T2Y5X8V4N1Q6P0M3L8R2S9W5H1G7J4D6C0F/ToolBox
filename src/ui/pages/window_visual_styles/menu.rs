//! Context menu construction and handling for the window visual styles ListView.
//!
//! Provides command IDs and helper functions to display the right-click
//! context menu for selected system processes.

use anyhow::Result;
use rust_i18n::t;
use winsafe::{HMENU, HWND, MenuItem, POINT, co};

use crate::ui::menu::UnattachedMenuGuard;

/// Command ID for applying basic visual style to the selected process window.
pub const IDM_VISUAL_STYLES_APPLY_BASIC: u16 = 3101;

/// Command ID for applying classic visual style to the selected process window.
pub const IDM_VISUAL_STYLES_APPLY_CLASSIC: u16 = 3102;

/// Construct and display the context menu at the specified screen coordinates.
///
/// Builds a transient popup menu managed by [`UnattachedMenuGuard`]. Once tracked and
/// dismissed (or if any operation fails early), the menu handle is automatically
/// destroyed upon scope exit via RAII.
pub(super) fn show_process_context_menu(
    parent_window_hwnd: &HWND,
    screen_position: POINT,
) -> Result<()> {
    let popup_menu_handle = HMENU::CreatePopupMenu()?;
    let popup_menu_guard = UnattachedMenuGuard::new(popup_menu_handle);

    popup_menu_guard.handle()?.append_item(&[
        MenuItem::Entry {
            cmd_id: IDM_VISUAL_STYLES_APPLY_BASIC,
            text: &t!("MENU_VISUAL_STYLES_APPLY_BASIC"),
        },
        MenuItem::Entry {
            cmd_id: IDM_VISUAL_STYLES_APPLY_CLASSIC,
            text: &t!("MENU_VISUAL_STYLES_APPLY_CLASSIC"),
        },
    ])?;

    popup_menu_guard.handle()?.TrackPopupMenu(
        co::TPM::LEFTALIGN | co::TPM::RIGHTBUTTON,
        screen_position,
        parent_window_hwnd,
    )?;

    // Scope exit here automatically triggers popup_menu_guard's Drop, cleanly destroying
    // the transient popup menu without manual invocation or leak risks on early return.
    Ok(())
}
