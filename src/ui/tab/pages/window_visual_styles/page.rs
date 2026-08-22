use winsafe::{gui, prelude::*};

use crate::ui;

#[derive(Clone)]
pub struct WindowVisualStylesPage {
    tab_page: gui::TabPage,
}

impl From<WindowVisualStylesPage> for gui::TabPage {
    fn from(window_visual_style_page_page: WindowVisualStylesPage) -> gui::TabPage {
        window_visual_style_page_page.tab_page.clone()
    }
}

impl WindowVisualStylesPage {
    pub fn new(parent_window: &(impl GuiParent + 'static)) -> Self {
        let tab_page = gui::TabPage::new(parent_window, gui::TabPageOpts::default());
        let window_visual_style_page_page = Self { tab_page };
        window_visual_style_page_page.setup_events();
        window_visual_style_page_page
    }

    fn setup_events(&self) {
        ui::tab::utils::setup_tab_page_background_events(&self.tab_page);
    }
}
