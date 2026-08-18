use winsafe::{co, gui, prelude::*};

use crate::ui;

#[derive(Clone)]
pub struct SettingsPage {
    tab_page: gui::TabPage,
    group_box: gui::Button,
}

impl From<SettingsPage> for gui::TabPage {
    fn from(settings_page: SettingsPage) -> gui::TabPage {
        settings_page.tab_page.clone()
    }
}

impl SettingsPage {
    pub fn new(parent_window: &(impl GuiParent + 'static)) -> Self {
        let tab_page = gui::TabPage::new(parent_window, gui::TabPageOpts::default());
        let group_box = gui::Button::new(
            &tab_page,
            gui::ButtonOpts {
                text: "TEST GROUPBOX TITLE",
                position: gui::dpi(10, 10),
                width: gui::dpi_x(240),
                height: gui::dpi_x(120),
                control_style: co::BS::GROUPBOX,
                ..Default::default()
            },
        );
        let settings_page = Self {
            tab_page,
            group_box,
        };
        settings_page.setup_events();
        settings_page
    }

    fn setup_events(&self) {
        ui::tab::tab_page_utils::setup_tab_page_background_events(&self.tab_page);
    }
}
