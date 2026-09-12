//! Settings page control construction.
//!
//! Each function creates exactly one control. All controls are created during
//! [`SettingsPage::new`] before the message loop starts — Win32 requires
//! controls to be created on the same thread as their parent window.

use rust_i18n::t;
use winsafe::{POINT, co, gui, prelude::*};

use super::{CheckboxId, layout::CheckboxLayoutCalculator};

/// Create the [`gui::TabPage`] that hosts all settings controls.
pub(super) fn create_tab_page(parent_window: &(impl GuiParent + 'static)) -> gui::TabPage {
    gui::TabPage::new(
        parent_window,
        gui::TabPageOpts {
            class_style: co::CS::HREDRAW | co::CS::VREDRAW,
            ..Default::default()
        },
    )
}

/// Create the group box frame that visually wraps the checkbox list.
pub(super) fn create_group_box(parent_window: &gui::TabPage) -> gui::Button {
    gui::Button::new(
        parent_window,
        gui::ButtonOpts {
            text: &t!("GROUP_BOX_SETTINGS_TITLE"),
            control_style: co::BS::GROUPBOX,
            ..Default::default()
        },
    )
}

/// Create the outer scrollable panel that clips and scrolls the content panel.
///
/// `WS_VSCROLL` adds the vertical scrollbar track. `WS_EX_COMPOSITED` enables
/// double-buffered painting on this window and all its children, which
/// eliminates the flicker visible during scroll operations.
pub(super) fn create_scrollable_panel(parent_window: &gui::TabPage) -> gui::WindowControl {
    gui::WindowControl::new(
        parent_window,
        gui::WindowControlOpts {
            style: co::WS::CHILD
                | co::WS::VISIBLE
                | co::WS::CLIPCHILDREN
                | co::WS::CLIPSIBLINGS
                | co::WS::VSCROLL,
            ex_style: co::WS_EX::COMPOSITED,
            class_style: co::CS::HREDRAW | co::CS::VREDRAW,
            class_bg_brush: gui::Brush::Color(co::COLOR::BTNFACE),
            ..Default::default()
        },
    )
}

/// Create the inner content panel that holds all checkboxes.
///
/// This panel is a direct child of the scrollable panel and is intentionally
/// taller than its parent when there are many checkboxes. Scrolling is
/// implemented by shifting this panel's Y position upward.
pub(super) fn create_content_panel(parent_window: &gui::WindowControl) -> gui::WindowControl {
    gui::WindowControl::new(
        parent_window,
        gui::WindowControlOpts {
            style: co::WS::CHILD | co::WS::VISIBLE | co::WS::CLIPCHILDREN | co::WS::CLIPSIBLINGS,
            class_style: co::CS::HREDRAW | co::CS::VREDRAW,
            class_bg_brush: gui::Brush::Color(co::COLOR::BTNFACE),
            ..Default::default()
        },
    )
}

/// Create all checkbox controls, one per [`CheckboxId`] variant, in display order.
///
/// Returns a `Vec` of `(id, button)` pairs so callers can look up the
/// [`CheckboxId`] for any given button handle and vice versa.
pub(super) fn create_checkboxes(
    content_panel: &gui::WindowControl,
) -> Vec<(CheckboxId, gui::Button)> {
    let layout_calculator = CheckboxLayoutCalculator::new();

    CheckboxId::all()
        .iter()
        .enumerate()
        .map(|(index, &checkbox_id)| {
            let POINT { x, y } = layout_calculator.calculate_checkbox_position(index);
            let checkbox = gui::Button::new(
                content_panel,
                gui::ButtonOpts {
                    text: &t!(checkbox_id.i18n_key()),
                    position: (x, y),
                    width: 250,
                    height: layout_calculator.checkbox_height(),
                    ctrl_id: checkbox_id.window_control_id(),
                    control_style: co::BS::AUTOCHECKBOX,
                    ..Default::default()
                },
            );
            (checkbox_id, checkbox)
        })
        .collect()
}

/// Create the "Select All / Deselect All" toggle button.
pub(super) fn create_button_select_all_toggle(parent_window: &gui::TabPage) -> gui::Button {
    gui::Button::new(
        parent_window,
        gui::ButtonOpts {
            text: &t!("BUTTON_SELECT_ALL_TOGGLE"),
            width: super::layout::BUTTON_WIDTH,
            height: super::layout::BUTTON_HEIGHT,
            ..Default::default()
        },
    )
}

/// Create the "Apply" button.
pub(super) fn create_button_apply(parent_window: &gui::TabPage) -> gui::Button {
    gui::Button::new(
        parent_window,
        gui::ButtonOpts {
            text: &t!("BUTTON_APPLY"),
            width: super::layout::BUTTON_WIDTH,
            height: super::layout::BUTTON_HEIGHT,
            ..Default::default()
        },
    )
}
