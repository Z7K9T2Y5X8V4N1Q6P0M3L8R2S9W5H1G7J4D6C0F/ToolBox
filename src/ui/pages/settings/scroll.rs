//! Vertical scroll event handlers for the settings page.
//!
//! Scrolling is implemented by shifting the content panel's Y position
//! rather than using a native scrolling window. This gives full control
//! over the scroll behavior and avoids artifacts from nested scroll windows.
//!
//! Two events drive scrolling:
//! - `WM_VSCROLL` on the scrollable panel — fired by the scrollbar track and arrows
//! - `WM_MOUSEWHEEL` on the content panel — fired when the mouse wheel is used

use winsafe::{
    AnyResult, HwndPlace, POINT, SCROLLINFO, SIZE, SystemParametersInfo, co, gui, msg, prelude::*,
};

/// Register the `WM_VSCROLL` handler on the scrollable panel.
///
/// Translates each scroll request variant into a new absolute scroll position
/// and delegates to [`apply_scroll_position`].
pub(super) fn setup_scroll_event(
    scrollable_panel: &gui::WindowControl,
    content_panel: &gui::WindowControl,
) {
    let cloned_content_panel = content_panel.clone();
    let cloned_scrollable_panel = scrollable_panel.clone();

    scrollable_panel
        .on()
        .wm_v_scroll(move |vertical_scroll_info| {
            let mut current_scroll_info = SCROLLINFO::default();
            current_scroll_info.fMask = co::SIF::ALL;
            cloned_scrollable_panel
                .hwnd()
                .GetScrollInfo(co::SBB::VERT, &mut current_scroll_info)?;

            let old_scroll_position = current_scroll_info.nPos;
            let scrollable_maximum =
                (current_scroll_info.nMax - current_scroll_info.nPage as i32).max(0);

            let scroll_line_height = gui::dpi_y(20);
            let new_scroll_position = match vertical_scroll_info.request {
                co::SB_REQ::LINEUP => (old_scroll_position - scroll_line_height).max(0),
                co::SB_REQ::LINEDOWN => {
                    (old_scroll_position + scroll_line_height).min(scrollable_maximum)
                }
                co::SB_REQ::PAGEUP => {
                    (old_scroll_position - current_scroll_info.nPage as i32).max(0)
                }
                co::SB_REQ::PAGEDOWN => {
                    (old_scroll_position + current_scroll_info.nPage as i32).min(scrollable_maximum)
                }
                co::SB_REQ::THUMBTRACK | co::SB_REQ::THUMBPOSITION => {
                    (vertical_scroll_info.scroll_box_pos as i32)
                        .min(scrollable_maximum)
                        .max(0)
                }
                co::SB_REQ::TOP => 0,
                co::SB_REQ::BOTTOM => scrollable_maximum,
                _ => old_scroll_position,
            };

            if new_scroll_position != old_scroll_position {
                apply_scroll_position(
                    &cloned_scrollable_panel,
                    &cloned_content_panel,
                    new_scroll_position,
                )?;
            }

            Ok(())
        });
}

/// Register the `WM_MOUSEWHEEL` handler on the content panel.
///
/// Mouse wheel messages are delivered to the control under the cursor,
/// which is the content panel rather than the scrollable panel. We translate
/// wheel deltas into synthetic `WM_VSCROLL` messages sent to the scrollable
/// panel so the existing scroll handler processes them uniformly.
pub(super) fn setup_mousewheel_event(
    scrollable_panel: &gui::WindowControl,
    content_panel: &gui::WindowControl,
) {
    let cloned_scrollable_panel = scrollable_panel.clone();
    content_panel.on().wm_mouse_wheel(move |mouse_wheel_info| {
        let wheel_distance = mouse_wheel_info.wheel_distance;

        let is_scrolling_up = wheel_distance > 0;
        let wheel_notches = (wheel_distance.abs() / 120).max(1) as u32;

        let (request, total_wheel_notches) = match get_wheel_scroll_lines() {
            Some(lines_per_wheel_notch) => {
                let request = if is_scrolling_up {
                    co::SB_REQ::LINEUP
                } else {
                    co::SB_REQ::LINEDOWN
                };
                (request, wheel_notches * lines_per_wheel_notch)
            }
            None => {
                let request = if is_scrolling_up {
                    co::SB_REQ::PAGEUP
                } else {
                    co::SB_REQ::PAGEDOWN
                };
                (request, wheel_notches)
            }
        };

        for _ in 0..total_wheel_notches {
            unsafe {
                cloned_scrollable_panel.hwnd().SendMessage(msg::WmVScroll {
                    request,
                    scroll_box_pos: 0,
                    hcontrol: None,
                });
            }
        }
        Ok(())
    });
}

/// Query the system preference for how many lines to scroll per wheel notch.
///
/// Returns `None` when the system is configured for whole-page scrolling
/// (`WHEEL_PAGESCROLL`), in which case the caller should use
/// `SB_PAGEUP` / `SB_PAGEDOWN` instead of `SB_LINEUP` / `SB_LINEDOWN`.
/// Falls back to 3 lines if the system call fails.
fn get_wheel_scroll_lines() -> Option<u32> {
    const WHEEL_PAGESCROLL: u32 = u32::MAX;

    let mut wheel_scroll_lines: u32 = 0;
    let fetch_scroll_lines_result = unsafe {
        SystemParametersInfo(
            co::SPI::GETWHEELSCROLLLINES,
            0,
            &mut wheel_scroll_lines,
            co::SPIF::NoValue,
        )
    };
    match fetch_scroll_lines_result {
        Ok(_) if wheel_scroll_lines == WHEEL_PAGESCROLL => None,
        Ok(_) => Some(wheel_scroll_lines),
        Err(_) => Some(3),
    }
}

/// Update the scrollbar thumb position and shift the content panel accordingly.
///
/// The content panel's Y position is set to `-new_scroll_position` so that
/// the portion of the content that should be visible is aligned with the top
/// of the scrollable panel's client area.
pub(super) fn apply_scroll_position(
    scrollable_panel: &gui::WindowControl,
    content_panel: &gui::WindowControl,
    new_scroll_position: i32,
) -> AnyResult<()> {
    let mut scroll_info = SCROLLINFO::default();
    scroll_info.fMask = co::SIF::POS;
    scroll_info.nPos = new_scroll_position;
    scrollable_panel
        .hwnd()
        .SetScrollInfo(co::SBB::VERT, &scroll_info, true);

    content_panel.hwnd().SetWindowPos(
        HwndPlace::None,
        POINT {
            x: 0,
            y: -new_scroll_position,
        },
        SIZE::default(),
        co::SWP::NOZORDER | co::SWP::NOCOPYBITS | co::SWP::NOSIZE,
    )?;

    Ok(())
}
