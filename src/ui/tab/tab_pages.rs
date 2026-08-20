use rust_i18n::t;
use winsafe::{HwndPlace, POINT, SIZE, co, gui, msg, prelude::*};

use super::{settings_page::SettingsPage, window_visual_styles_page::WindowVisualStylesPage};

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
            Self::create_tab_control(parent_window, &settings_page, &window_visual_styles_page);

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
        self.resize_tab_control(window_client_width, window_client_height)?;
        self.resize_current_tab_page()?;
        Ok(())
    }

    pub fn update_tab_control_titles(&self) -> winsafe::AnyResult<()> {
        let tab_control_titles = Self::get_tab_control_titles();
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

    fn create_tab_control(
        parent_window: &(impl GuiParent + 'static),
        settings_page: &SettingsPage,
        window_visual_styles_page: &WindowVisualStylesPage,
    ) -> gui::Tab {
        let tab_control_titles = Self::get_tab_control_titles();

        gui::Tab::new(
            parent_window,
            gui::TabOpts {
                pages: &[
                    (&tab_control_titles[0], settings_page.clone().into()),
                    (
                        &tab_control_titles[1],
                        window_visual_styles_page.clone().into(),
                    ),
                ],
                ..Default::default()
            },
        )
    }

    fn get_tab_control_titles() -> Vec<String> {
        vec![
            t!("TAB_SETTINGS").to_string(),
            t!("TAB_WINDOW_VISUAL_STYLES").to_string(),
        ]
    }

    fn resize_tab_control(
        &self,
        window_client_width: i32,
        window_client_height: i32,
    ) -> winsafe::AnyResult<()> {
        let tab_control_margin = gui::dpi_x(10);
        let tab_control_size = SIZE {
            cx: window_client_width - (tab_control_margin * 2),
            cy: window_client_height - (tab_control_margin * 2),
        };

        self.tab_control.hwnd().SetWindowPos(
            HwndPlace::None,
            POINT {
                x: tab_control_margin,
                y: tab_control_margin,
            },
            tab_control_size,
            co::SWP::NOZORDER,
        )?;

        Ok(())
    }

    fn resize_current_tab_page(&self) -> winsafe::AnyResult<()> {
        let current_selected_tab_control_item_index = match self.tab_control.items().selected() {
            Some(selected_tab_control_item) => selected_tab_control_item.index() as usize,
            None => return Ok(()),
        };

        let target_tab_page = match self.tab_pages.get(current_selected_tab_control_item_index) {
            Some(target_tab_page) => target_tab_page,
            None => return Ok(()),
        };

        let tab_page_screen_to_client_rect = self.calculate_tab_page_rect()?;
        let calculated_tab_page_content_size = SIZE {
            cx: tab_page_screen_to_client_rect.right - tab_page_screen_to_client_rect.left,
            cy: tab_page_screen_to_client_rect.bottom - tab_page_screen_to_client_rect.top,
        };

        target_tab_page.hwnd().SetWindowPos(
            HwndPlace::None,
            POINT {
                x: tab_page_screen_to_client_rect.left,
                y: tab_page_screen_to_client_rect.top,
            },
            calculated_tab_page_content_size,
            co::SWP::NOZORDER,
        )?;

        Ok(())
    }

    fn calculate_tab_page_rect(&self) -> winsafe::AnyResult<winsafe::RECT> {
        let tab_control_hwnd = self.tab_control.hwnd();
        let tab_control_parent_hwnd = tab_control_hwnd.GetParent()?;

        let mut tab_page_screen_to_client_rect =
            tab_control_parent_hwnd.ScreenToClientRc(tab_control_hwnd.GetWindowRect()?)?;

        unsafe {
            tab_control_hwnd.SendMessage(msg::TcmAdjustRect {
                display_rect: false,
                rect: &mut tab_page_screen_to_client_rect,
            });
        }

        Ok(tab_page_screen_to_client_rect)
    }
}
