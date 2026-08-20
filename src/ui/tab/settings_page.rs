use rust_i18n::t;
use winsafe::{HwndPlace, POINT, SIZE, co, gui, prelude::*};

use crate::ui;

const GROUP_BOX_SETTINGS_TITLE: &str = "GROUP_BOX_SETTINGS_TITLE";

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
        let tab_page = Self::create_tab_page(parent_window);
        let group_box = Self::create_group_box(&tab_page);

        let settings_page = Self {
            tab_page,
            group_box,
        };
        settings_page.setup_events();
        settings_page
    }

    pub fn update_texts(&self) -> winsafe::AnyResult<()> {
        self.group_box
            .hwnd()
            .SetWindowText(&t!(GROUP_BOX_SETTINGS_TITLE))?;
        Ok(())
    }

    fn create_tab_page(parent_window: &(impl GuiParent + 'static)) -> gui::TabPage {
        gui::TabPage::new(
            parent_window,
            gui::TabPageOpts {
                class_style: co::CS::HREDRAW | co::CS::VREDRAW,
                ..Default::default()
            },
        )
    }

    fn create_group_box(parent_window: &gui::TabPage) -> gui::Button {
        gui::Button::new(
            parent_window,
            gui::ButtonOpts {
                text: &t!(GROUP_BOX_SETTINGS_TITLE),
                control_style: co::BS::GROUPBOX,
                ..Default::default()
            },
        )
    }

    fn setup_events(&self) {
        ui::tab::tab_page_utils::setup_tab_page_background_events(&self.tab_page);
        self.setup_resize_event();
    }

    fn setup_resize_event(&self) {
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
