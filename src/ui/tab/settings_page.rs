use winsafe::{HwndPlace, POINT, SIZE, co, gui, prelude::*};

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
        let tab_page = gui::TabPage::new(
            parent_window,
            gui::TabPageOpts {
                class_style: co::CS::HREDRAW | co::CS::VREDRAW,
                ..Default::default()
            },
        );
        let group_box = gui::Button::new(
            &tab_page,
            gui::ButtonOpts {
                text: "TEST GROUPBOX TITLE",
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

        let group_box = self.group_box.clone();

        self.tab_page.on().wm_size(move |size_info| {
            let group_box_margin = gui::dpi_x(10);
            let tab_page_client_width = size_info.client_area.cx;
            let tab_page_client_height = size_info.client_area.cy;

            group_box.hwnd().SetWindowPos(
                HwndPlace::None,
                POINT {
                    x: group_box_margin,
                    y: group_box_margin,
                },
                SIZE {
                    cx: tab_page_client_width - 2 * group_box_margin,
                    cy: tab_page_client_height - 2 * group_box_margin,
                },
                co::SWP::NOZORDER | co::SWP::NOCOPYBITS,
            )?;

            Ok(())
        });
    }
}
