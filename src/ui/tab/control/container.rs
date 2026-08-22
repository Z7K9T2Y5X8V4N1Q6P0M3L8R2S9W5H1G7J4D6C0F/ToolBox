use super::{builder, layout};
use crate::ui::tab::pages::{settings::SettingsPage, window_visual_styles::WindowVisualStylesPage};
use winsafe::{gui, prelude::*};

#[derive(Clone)]
pub struct TabPages {
    tab_control: gui::Tab,
    tab_pages: Vec<gui::TabPage>,
    settings_page: SettingsPage,
    window_visual_styles_page: WindowVisualStylesPage,
}

impl TabPages {
    pub fn new(parent_window: &(impl GuiParent + 'static)) -> Self {
        let settings_page = SettingsPage::new(parent_window);
        let window_visual_styles_page = WindowVisualStylesPage::new(parent_window);

        let tab_pages = vec![
            settings_page.clone().into(),
            window_visual_styles_page.clone().into(),
        ];

        let tab_control =
            builder::create_tab_control(parent_window, &settings_page, &window_visual_styles_page);

        Self {
            tab_control,
            tab_pages,
            settings_page,
            window_visual_styles_page,
        }
    }

    pub fn resize(
        &self,
        window_client_width: i32,
        window_client_height: i32,
    ) -> winsafe::AnyResult<()> {
        layout::resize_tab_control(&self.tab_control, window_client_width, window_client_height)?;
        layout::resize_current_tab_page(&self.tab_control, &self.tab_pages)?;
        Ok(())
    }

    pub fn update_tab_control_titles(&self) -> winsafe::AnyResult<()> {
        let tab_control_titles = builder::get_tab_control_titles();
        for (tab_control_index, tab_control_title) in tab_control_titles.iter().enumerate() {
            let target_tab_control_item = self.tab_control.items().get(tab_control_index as u32);
            target_tab_control_item.set_text(tab_control_title)?;
        }

        Ok(())
    }

    pub fn update_page_contents(&self) -> winsafe::AnyResult<()> {
        self.settings_page.update_texts()?;

        Ok(())
    }
}
