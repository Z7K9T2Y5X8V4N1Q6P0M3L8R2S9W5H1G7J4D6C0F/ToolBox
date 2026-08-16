use winsafe::{gui, prelude::*};

#[derive(Clone)]
pub struct WindowVisualStylesPage {
    tab_page: gui::TabPage,
}

impl From<WindowVisualStylesPage> for gui::TabPage {
    fn from(page: WindowVisualStylesPage) -> gui::TabPage {
        page.tab_page.clone()
    }
}

impl WindowVisualStylesPage {
    pub fn new(parent_window: &(impl GuiParent + 'static)) -> Self {
        let tab_page = gui::TabPage::new(parent_window, gui::TabPageOpts::default());
        Self { tab_page }
    }
}
