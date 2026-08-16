use winsafe::{gui, prelude::*};

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
        Self { tab_page }
    }
}
