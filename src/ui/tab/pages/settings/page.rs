use rust_i18n::t;
use winsafe::{co, gui, prelude::*};

use crate::ui::tab::pages::settings::layout::{BUTTON_HEIGHT, BUTTON_WIDTH};

use super::events;

#[derive(Clone)]
pub struct SettingsPage {
    tab_page: gui::TabPage,
    group_box: gui::Button,
    button_select_all_toggle: gui::Button,
    button_apply: gui::Button,
}

impl From<SettingsPage> for gui::TabPage {
    fn from(settings_page: SettingsPage) -> gui::TabPage {
        settings_page.tab_page.clone()
    }
}

impl SettingsPage {
    pub fn new(parent_window: &(impl GuiParent + 'static)) -> Self {
        let tab_page = Self::create_tab_page(parent_window);
        let group_box = Self::create_group_box(&tab_page);
        let button_select_all_toggle = Self::create_button_select_all_toggle(&tab_page);
        let button_apply = Self::create_button_apply(&tab_page);

        events::setup_all_events(
            &tab_page,
            &group_box,
            &button_select_all_toggle,
            &button_apply,
        );

        Self {
            tab_page,
            group_box,
            button_select_all_toggle,
            button_apply,
        }
    }

    pub fn update_texts(&self) -> winsafe::AnyResult<()> {
        self.group_box
            .hwnd()
            .SetWindowText(&t!("GROUP_BOX_SETTINGS_TITLE"))?;

        self.button_select_all_toggle
            .hwnd()
            .SetWindowText(&t!("BUTTON_SELECT_ALL_TOGGLE"))?;

        self.button_apply
            .hwnd()
            .SetWindowText(&t!("BUTTON_APPLY"))?;
        Ok(())
    }

    fn create_tab_page(parent_window: &(impl GuiParent + 'static)) -> gui::TabPage {
        gui::TabPage::new(
            parent_window,
            gui::TabPageOpts {
                class_style: co::CS::HREDRAW | co::CS::VREDRAW,
                ..Default::default()
            },
        )
    }

    fn create_group_box(parent_window: &gui::TabPage) -> gui::Button {
        gui::Button::new(
            parent_window,
            gui::ButtonOpts {
                text: &t!("GROUP_BOX_SETTINGS_TITLE"),
                control_style: co::BS::GROUPBOX,
                ..Default::default()
            },
        )
    }

    fn create_button_select_all_toggle(parent_window: &gui::TabPage) -> gui::Button {
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

    fn create_button_apply(parent_window: &gui::TabPage) -> gui::Button {
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
}
