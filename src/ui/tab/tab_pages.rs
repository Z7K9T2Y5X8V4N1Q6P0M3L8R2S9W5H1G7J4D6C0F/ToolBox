use rust_i18n::t;
use winsafe::{POINT, SIZE, co, gui, msg, prelude::*};

use super::{settings_page::SettingsPage, window_visual_styles_page::WindowVisualStylesPage};

#[derive(Clone)]
pub struct TabPages {
    tab_control: gui::Tab,
    tab_pages: Vec<gui::TabPage>,
}

impl TabPages {
    pub fn new(parent_window: &(impl GuiParent + 'static)) -> Self {
        let settings_page = SettingsPage::new(parent_window);
        let window_visual_styles_page = WindowVisualStylesPage::new(parent_window);

        let tab_pages = vec![
            settings_page.clone().into(),
            window_visual_styles_page.clone().into(),
        ];

        let tab_control_titles = Self::get_tab_control_titles();
        let tab_control = gui::Tab::new(
            parent_window,
            gui::TabOpts {
                position: gui::dpi(10, 10),
                size: gui::dpi(280, 480),
                pages: &[
                    (&tab_control_titles[0], settings_page.into()),
                    (&tab_control_titles[1], window_visual_styles_page.into()),
                ],
                ..Default::default()
            },
        );

        Self {
            tab_control,
            tab_pages,
        }
    }

    pub fn resize(&self, client_width: i32, client_height: i32) -> winsafe::AnyResult<()> {
        let tab_control_margin = gui::dpi_x(10);
        let tab_control_width = client_width - (tab_control_margin * 2);
        let tab_control_height = client_height - (tab_control_margin * 2);

        self.tab_control.hwnd().SetWindowPos(
            winsafe::HwndPlace::None,
            POINT {
                x: tab_control_margin,
                y: tab_control_margin,
            },
            SIZE {
                cx: tab_control_width,
                cy: tab_control_height,
            },
            co::SWP::NOZORDER,
        )?;

        self.resize_current_tab_page()?;

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

        let tab_control_hwnd = self.tab_control.hwnd();
        let tab_control_parent_hwnd = tab_control_hwnd.GetParent()?;
        let mut tab_page_content_screen_to_client_rect =
            tab_control_parent_hwnd.ScreenToClientRc(tab_control_hwnd.GetWindowRect()?)?;
        unsafe {
            tab_control_hwnd.SendMessage(msg::TcmAdjustRect {
                display_rect: false,
                rect: &mut tab_page_content_screen_to_client_rect,
            });
        }

        let calculated_tab_page_content_width = tab_page_content_screen_to_client_rect.right
            - tab_page_content_screen_to_client_rect.left;
        let calculated_tab_page_content_height = tab_page_content_screen_to_client_rect.bottom
            - tab_page_content_screen_to_client_rect.top;
        target_tab_page.hwnd().SetWindowPos(
            winsafe::HwndPlace::None,
            POINT {
                x: tab_page_content_screen_to_client_rect.left,
                y: tab_page_content_screen_to_client_rect.top,
            },
            SIZE {
                cx: calculated_tab_page_content_width,
                cy: calculated_tab_page_content_height,
            },
            co::SWP::NOZORDER,
        )?;

        Ok(())
    }

    fn get_tab_control_titles() -> Vec<String> {
        vec![
            t!("TAB_SETTINGS").to_string(),
            t!("TAB_WINDOW_VISUAL_STYLES").to_string(),
        ]
    }

    pub fn update_tab_control_titles(&self) -> winsafe::AnyResult<()> {
        let tab_control_titles = Self::get_tab_control_titles();

        for (tab_control_index, tab_control_title) in tab_control_titles.iter().enumerate() {
            let target_tab_control_item = self.tab_control.items().get(tab_control_index as u32);
            target_tab_control_item.set_text(tab_control_title)?;
        }

        Ok(())
    }
}
