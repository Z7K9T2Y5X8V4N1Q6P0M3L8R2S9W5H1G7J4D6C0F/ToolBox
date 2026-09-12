//! Tab control and tab page layout utilities.
//!
//! All sizing and positioning logic for the tab control widget and its hosted
//! pages lives here. Also provides shared helpers used by page-level event
//! handlers for control repositioning and Z-order management.

use winsafe::{
    AnyResult, HBRUSH, HWND, HwndPlace, POINT, RECT, SIZE, co, gui, msg,
    prelude::{GuiEventsWindow, GuiWindow},
};

const TAB_CONTROL_MARGIN: i32 = 10;

/// Resize the tab control to fill the main window client area with a uniform margin.
pub(crate) fn resize_tab_control(
    tab_control: &gui::Tab,
    window_client_width: i32,
    window_client_height: i32,
) -> AnyResult<()> {
    let tab_control_margin = gui::dpi_x(TAB_CONTROL_MARGIN);
    let tab_control_size = SIZE {
        cx: window_client_width - (tab_control_margin * 2),
        cy: window_client_height - (tab_control_margin * 2),
    };

    tab_control.hwnd().SetWindowPos(
        HwndPlace::None,
        POINT {
            x: tab_control_margin,
            y: tab_control_margin,
        },
        tab_control_size,
        co::SWP::NOZORDER,
    )?;

    Ok(())
}

/// Resize the currently selected tab page to fill the tab control's content area.
///
/// Only the visible page is resized. If no tab is selected, this is a no-op.
pub(crate) fn resize_current_tab_page(
    tab_control: &gui::Tab,
    tab_pages: &[gui::TabPage],
) -> AnyResult<()> {
    let Some(selected_tab_control_item) = tab_control.items().selected() else {
        return Ok(());
    };

    let Some(target_tab_page) = tab_pages.get(selected_tab_control_item.index() as usize) else {
        return Ok(());
    };

    let tab_page_rect = calculate_tab_page_rect(tab_control.hwnd())?;

    let calculated_tab_page_content_size = SIZE {
        cx: tab_page_rect.right - tab_page_rect.left,
        cy: tab_page_rect.bottom - tab_page_rect.top,
    };

    target_tab_page.hwnd().SetWindowPos(
        HwndPlace::None,
        POINT {
            x: tab_page_rect.left,
            y: tab_page_rect.top,
        },
        calculated_tab_page_content_size,
        co::SWP::NOZORDER,
    )?;

    Ok(())
}

/// Calculate the content rectangle of a tab page in the parent's client coordinates.
///
/// The tab control header (the row of tab title buttons) occupies some space
/// at the top of the tab control. `TCM_ADJUSTRECT` with `display_rect = false`
/// converts the full tab control rect into the smaller content area beneath
/// the header, which is where the tab pages actually live.
pub(crate) fn calculate_tab_page_rect(tab_control_hwnd: &HWND) -> AnyResult<RECT> {
    let tab_control_parent_hwnd = tab_control_hwnd.GetParent()?;

    let mut tab_page_rect =
        tab_control_parent_hwnd.ScreenToClientRc(tab_control_hwnd.GetWindowRect()?)?;

    unsafe {
        tab_control_hwnd.SendMessage(msg::TcmAdjustRect {
            display_rect: false,
            rect: &mut tab_page_rect,
        });
    }

    Ok(tab_page_rect)
}

/// Reposition and resize a control in a single [`SetWindowPos`] call.
///
/// Uses `SWP_NOZORDER | SWP_NOCOPYBITS` to avoid unnecessary repaints
/// and Z-order changes.
pub fn reposition_and_resize_control(
    control_hwnd: &HWND,
    position: POINT,
    size: SIZE,
) -> AnyResult<()> {
    control_hwnd.SetWindowPos(
        HwndPlace::None,
        position,
        size,
        co::SWP::NOZORDER | co::SWP::NOCOPYBITS,
    )?;
    Ok(())
}

/// Bring a control to the top of the Z-order among its siblings.
///
/// Used to ensure the scrollable panel renders above the group box frame
/// so its scrollbar remains clickable.
pub fn bring_control_to_top(control_hwnd: &HWND) -> AnyResult<()> {
    control_hwnd.SetWindowPos(
        HwndPlace::Place(co::HWND_PLACE::TOP),
        POINT::default(),
        SIZE::default(),
        co::SWP::NOMOVE | co::SWP::NOSIZE,
    )?;
    Ok(())
}

/// Register a WM_ERASEBKGND handler that paints the tab page background
/// with the system button face color.
///
/// Without this, tab pages may show a white or transparent background
/// because WinSafe's [`gui::TabPage`] does not paint its own background.
/// This must be called during page construction, before the message loop starts.
pub fn paint_tab_page_background(tab_page: &gui::TabPage) {
    let cloned_tab_page = tab_page.clone();
    tab_page.on().wm_erase_bkgnd(move |erase_bkgnd_params| {
        let tab_page_client_rect = cloned_tab_page.hwnd().GetClientRect()?;
        let background_brush = HBRUSH::GetSysColorBrush(co::COLOR::BTNFACE)?;
        erase_bkgnd_params
            .hdc
            .FillRect(tab_page_client_rect, &background_brush)?;
        Ok(1)
    });
}
