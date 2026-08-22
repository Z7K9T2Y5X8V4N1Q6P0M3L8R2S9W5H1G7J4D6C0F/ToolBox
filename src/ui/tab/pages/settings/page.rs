use rust_i18n::t;
use winsafe::{gui, prelude::*};

use super::{builder, events};

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
        let tab_page = builder::create_tab_page(parent_window);
        let group_box = builder::create_group_box(&tab_page);
        let button_select_all_toggle = builder::create_button_select_all_toggle(&tab_page);
        let button_apply = builder::create_button_apply(&tab_page);

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
}
