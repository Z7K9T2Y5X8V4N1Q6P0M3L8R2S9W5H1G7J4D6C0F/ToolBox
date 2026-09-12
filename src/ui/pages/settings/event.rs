//! Settings page event registration.
//!
//! [`setup_all_events`] is the single entry point called from [`SettingsPage::new`].
//! It wires up every event handler for the settings page in the correct order.

use winsafe::{AnyResult, HwndPlace, POINT, SCROLLINFO, SIZE, co, gui, msg, prelude::*};

use crate::ui::{statusbar, tab};

use super::{
    CheckboxId,
    layout::{CheckboxLayoutCalculator, SettingsPageLayout},
    scroll,
};

/// Wire up all event handlers for the settings page.
///
/// Must be called once during [`SettingsPage::new`], after all controls
/// are constructed but before the message loop starts.
pub(super) fn setup_all_events(
    tab_page: &gui::TabPage,
    group_box: &gui::Button,
    scrollable_panel: &gui::WindowControl,
    content_panel: &gui::WindowControl,
    checkboxes: &[(CheckboxId, gui::Button)],
    button_select_all_toggle: &gui::Button,
    button_apply: &gui::Button,
    status_bar: &gui::StatusBar,
) {
    tab::paint_tab_page_background(tab_page);
    setup_resize_event(
        tab_page,
        group_box,
        scrollable_panel,
        content_panel,
        checkboxes,
        button_select_all_toggle,
        button_apply,
    );

    scroll::setup_scroll_event(scrollable_panel, content_panel);
    scroll::setup_mousewheel_event(scrollable_panel, content_panel);

    setup_button_select_all_toggle_event(button_select_all_toggle, checkboxes);
    setup_button_apply_event(button_apply);

    statusbar::register_hover_events(checkboxes, status_bar);
}

// ---------------------------------------------------------------------------
// Resize
// ---------------------------------------------------------------------------

/// Register the `WM_SIZE` handler that repositions all controls on the page.
fn setup_resize_event(
    tab_page: &gui::TabPage,
    group_box: &gui::Button,
    scrollable_panel: &gui::WindowControl,
    content_panel: &gui::WindowControl,
    checkboxes: &[(CheckboxId, gui::Button)],
    button_select_all_toggle: &gui::Button,
    button_apply: &gui::Button,
) {
    let cloned_group_box = group_box.clone();
    let cloned_scrollable_panel = scrollable_panel.clone();
    let cloned_content_panel = content_panel.clone();
    let cloned_checkboxes: Vec<gui::Button> = checkboxes
        .iter()
        .map(|(_, checkbox)| checkbox.clone())
        .collect();
    let cloned_button_select_all_toggle = button_select_all_toggle.clone();
    let cloned_button_apply = button_apply.clone();

    tab_page.on().wm_size(move |size_info| {
        // The scrollable panel must sit above the group box in the Z-order so
        // its scrollbar track remains clickable and is not obscured by the frame.
        tab::bring_control_to_top(cloned_scrollable_panel.hwnd())?;

        let settings_page_layout =
            SettingsPageLayout::calculate(size_info.client_area.cx, size_info.client_area.cy);

        tab::reposition_and_resize_control(
            cloned_group_box.hwnd(),
            settings_page_layout.group_box_position,
            settings_page_layout.group_box_size,
        )?;

        tab::reposition_and_resize_control(
            cloned_scrollable_panel.hwnd(),
            settings_page_layout.scrollable_panel_position,
            settings_page_layout.scrollable_panel_size,
        )?;

        let layout_calculator = CheckboxLayoutCalculator::new();
        let content_panel_width = settings_page_layout.scrollable_panel_size.cx;
        let content_panel_total_height =
            layout_calculator.calculate_total_content_height(cloned_checkboxes.len());
        let scrollable_panel_visible_height = settings_page_layout.scrollable_panel_size.cy;

        tab::reposition_and_resize_control(
            cloned_content_panel.hwnd(),
            POINT { x: 0, y: 0 },
            SIZE {
                cx: content_panel_width,
                cy: content_panel_total_height,
            },
        )?;

        let checkbox_width = layout_calculator.calculate_checkbox_width(content_panel_width);
        let checkbox_height = layout_calculator.checkbox_height();
        for (index, checkbox_button) in cloned_checkboxes.iter().enumerate() {
            let checkbox_position = layout_calculator.calculate_checkbox_position(index);
            checkbox_button.hwnd().SetWindowPos(
                HwndPlace::None,
                checkbox_position,
                SIZE {
                    cx: checkbox_width,
                    cy: checkbox_height,
                },
                co::SWP::NOZORDER | co::SWP::NOCOPYBITS,
            )?;
        }

        update_scrollbar_range(
            &cloned_scrollable_panel,
            &cloned_content_panel,
            scrollable_panel_visible_height,
            content_panel_total_height,
        )?;

        tab::reposition_and_resize_control(
            cloned_button_apply.hwnd(),
            settings_page_layout.button_apply_position,
            settings_page_layout.button_apply_size,
        )?;

        tab::reposition_and_resize_control(
            cloned_button_select_all_toggle.hwnd(),
            settings_page_layout.button_select_all_toggle_position,
            settings_page_layout.button_select_all_toggle_size,
        )?;

        Ok(())
    });
}

/// Recalculate the scrollbar range and page size after the panel is resized.
///
/// If all checkboxes fit within the visible height, the scrollbar is hidden
/// and the content panel is scrolled back to the top. Otherwise the scrollbar
/// range is set to `[0, content_height - 1]` with the visible height as the
/// page size, and the previous scroll position is clamped to the new maximum.
fn update_scrollbar_range(
    scrollable_panel: &gui::WindowControl,
    content_panel: &gui::WindowControl,
    scrollable_panel_visible_height: i32,
    content_panel_total_height: i32,
) -> AnyResult<()> {
    let is_needs_scrollbar = content_panel_total_height > scrollable_panel_visible_height;
    scrollable_panel
        .hwnd()
        .ShowScrollBar(co::SBB::VERT, is_needs_scrollbar)
        .ok();

    if is_needs_scrollbar {
        let scrollable_maximum =
            (content_panel_total_height - scrollable_panel_visible_height).max(0);

        let mut scroll_info_position = SCROLLINFO::default();
        scroll_info_position.fMask = co::SIF::POS;
        scrollable_panel
            .hwnd()
            .GetScrollInfo(co::SBB::VERT, &mut scroll_info_position)?;
        let old_vertical_scroll_position = scroll_info_position.nPos;
        let clamped_old_vertical_scroll_position =
            old_vertical_scroll_position.min(scrollable_maximum).max(0);

        let mut scroll_info = SCROLLINFO::default();
        scroll_info.fMask = co::SIF::RANGE | co::SIF::PAGE;
        scroll_info.nMin = 0;
        scroll_info.nMax = content_panel_total_height - 1;
        scroll_info.nPage = scrollable_panel_visible_height as u32;

        scrollable_panel
            .hwnd()
            .SetScrollInfo(co::SBB::VERT, &scroll_info, true);

        scroll::apply_scroll_position(
            scrollable_panel,
            content_panel,
            clamped_old_vertical_scroll_position,
        )?;
    } else {
        // Reset SCROLLINFO properties to zero to invalidate the scroll range.
        // Hiding the scrollbar via ShowScrollBar alone leaves the internal range
        // intact, which causes synthetic WM_VSCROLL messages from mouse wheel
        // events to continue scrolling an invisible track.
        let mut scroll_info = SCROLLINFO::default();
        scroll_info.fMask = co::SIF::RANGE | co::SIF::PAGE | co::SIF::POS;
        scroll_info.nMin = 0;
        scroll_info.nMax = 0;
        scroll_info.nPage = 0;
        scroll_info.nPos = 0;
        scrollable_panel
            .hwnd()
            .SetScrollInfo(co::SBB::VERT, &scroll_info, true);

        scroll::apply_scroll_position(scrollable_panel, content_panel, 0)?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Button events
// ---------------------------------------------------------------------------

/// Register the click handler for the "Select All / Deselect All" toggle button.
///
/// If every checkbox is currently checked, all are unchecked. Otherwise all
/// are checked. The state is read and written via `BM_GETCHECK` / `BM_SETCHECK`
/// rather than tracking it manually so it always reflects the true Win32 state.
fn setup_button_select_all_toggle_event(
    button_select_all_toggle: &gui::Button,
    checkboxes: &[(CheckboxId, gui::Button)],
) {
    let checkboxes_buttons: Vec<gui::Button> = checkboxes
        .iter()
        .map(|(_, checkbox)| checkbox.clone())
        .collect();

    button_select_all_toggle.on().bn_clicked(move || {
        let all_currently_checked = checkboxes_buttons.iter().all(|checkbox_button| {
            let check_state = unsafe { checkbox_button.hwnd().SendMessage(msg::BmGetCheck {}) };
            check_state == co::BST::CHECKED
        });

        let target_check_state = if all_currently_checked {
            co::BST::UNCHECKED
        } else {
            co::BST::CHECKED
        };

        for checkbox_button in &checkboxes_buttons {
            unsafe {
                checkbox_button.hwnd().SendMessage(msg::BmSetCheck {
                    state: target_check_state,
                });
            }
        }

        Ok(())
    });
}

/// Register the click handler for the "Apply" button.
///
/// Currently a no-op placeholder — actual registry write logic will be
/// added here when the apply functionality is implemented.
fn setup_button_apply_event(button_apply: &gui::Button) {
    button_apply.on().bn_clicked(move || Ok(()));
}
