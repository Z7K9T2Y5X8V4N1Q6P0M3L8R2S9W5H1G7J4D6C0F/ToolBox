use rust_i18n::t;
use winsafe::{co, gui, prelude::*};

use super::layout::{BUTTON_HEIGHT, BUTTON_WIDTH};

pub(super) fn create_tab_page(parent_window: &(impl GuiParent + 'static)) -> gui::TabPage {
    gui::TabPage::new(
        parent_window,
        gui::TabPageOpts {
            class_style: co::CS::HREDRAW | co::CS::VREDRAW,
            ..Default::default()
        },
    )
}

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

pub(super) fn create_button_select_all_toggle(parent_window: &gui::TabPage) -> gui::Button {
    gui::Button::new(
        parent_window,
        gui::ButtonOpts {
            text: &t!("BUTTON_SELECT_ALL_TOGGLE"),
            width: BUTTON_WIDTH,
            height: BUTTON_HEIGHT,
            ..Default::default()
        },
    )
}

pub(super) fn create_button_apply(parent_window: &gui::TabPage) -> gui::Button {
    gui::Button::new(
        parent_window,
        gui::ButtonOpts {
            text: &t!("BUTTON_APPLY"),
            width: BUTTON_WIDTH,
            height: BUTTON_HEIGHT,
            ..Default::default()
        },
    )
}
