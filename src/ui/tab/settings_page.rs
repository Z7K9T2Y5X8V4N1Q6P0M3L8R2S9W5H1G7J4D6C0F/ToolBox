use winsafe::{gui, prelude::*};

use crate::ui;

#[derive(Clone)]
pub struct SettingsPage {
    tab_page: gui::TabPage,
}

impl From<SettingsPage> for gui::TabPage {
    fn from(settings_page: SettingsPage) -> gui::TabPage {
        settings_page.tab_page.clone()
    }
}

impl SettingsPage {
    pub fn new(parent_window: &(impl GuiParent + 'static)) -> Self {
        let tab_page = gui::TabPage::new(parent_window, gui::TabPageOpts::default());
        let settings_page = Self { tab_page };
        settings_page.setup_events();
        settings_page
    }

    fn setup_events(&self) {
        ui::tab::tab_page_utils::setup_tab_page_background_events(&self.tab_page);
    }
}
