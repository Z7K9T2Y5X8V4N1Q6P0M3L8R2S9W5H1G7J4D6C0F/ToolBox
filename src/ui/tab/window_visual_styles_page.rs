use winsafe::{HBRUSH, co, gui, prelude::*};

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
        let cloned_tab_page_for_background = self.tab_page.clone();
        self.tab_page
            .on()
            .wm_erase_bkgnd(move |erase_bkgnd_params| {
                let tab_page_content_client_rect =
                    cloned_tab_page_for_background.hwnd().GetClientRect()?;
                let background_brush = HBRUSH::GetSysColorBrush(co::COLOR::BTNFACE)?;
                erase_bkgnd_params
                    .hdc
                    .FillRect(tab_page_content_client_rect, &background_brush)?;
                Ok(1)
            });
    }
}
