use rust_i18n::t;
use winsafe::{HwndPlace, POINT, SIZE, co, gui, prelude::*};

use crate::ui;

const GROUP_BOX_SETTINGS_TITLE: &str = "GROUP_BOX_SETTINGS_TITLE";
const BUTTON_SELECT_ALL_TOGGLE_TEXT: &str = "BUTTON_SELECT_ALL_TOGGLE";
const BUTTON_APPLY_TEXT: &str = "BUTTON_APPLY";

#[derive(Clone)]
pub struct SettingsPage {
    tab_page: gui::TabPage,
    group_box: gui::Button,
    button_select_all_toggle: gui::Button,
    button_apply: gui::Button,
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
        let button_select_all_toggle = Self::create_button_select_all_toggle(&tab_page);
        let button_apply = Self::create_button_apply(&tab_page);

        let settings_page = Self {
            tab_page,
            group_box,
            button_select_all_toggle,
            button_apply,
        };
        settings_page.setup_events();
        settings_page
    }

    pub fn update_texts(&self) -> winsafe::AnyResult<()> {
        self.group_box
            .hwnd()
            .SetWindowText(&t!(GROUP_BOX_SETTINGS_TITLE))?;

        self.button_select_all_toggle
            .hwnd()
            .SetWindowText(&t!(BUTTON_SELECT_ALL_TOGGLE_TEXT))?;

        self.button_apply
            .hwnd()
            .SetWindowText(&t!(BUTTON_APPLY_TEXT))?;
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

    fn create_button_select_all_toggle(parent_window: &gui::TabPage) -> gui::Button {
        gui::Button::new(
            parent_window,
            gui::ButtonOpts {
                text: &t!(BUTTON_SELECT_ALL_TOGGLE_TEXT),
                width: 75,
                height: 25,
                ..Default::default()
            },
        )
    }

    fn create_button_apply(parent_window: &gui::TabPage) -> gui::Button {
        gui::Button::new(
            parent_window,
            gui::ButtonOpts {
                text: &t!(BUTTON_APPLY_TEXT),
                width: 75,
                height: 25,
                ..Default::default()
            },
        )
    }

    fn setup_events(&self) {
        ui::tab::tab_page_utils::setup_tab_page_background_events(&self.tab_page);
        self.setup_resize_event();
        self.setup_button_select_all_toggle_event();
        self.setup_button_apply_event();
    }

    fn setup_resize_event(&self) {
        let group_box = self.group_box.clone();
        let button_select_all_toggle = self.button_select_all_toggle.clone();
        let button_apply = self.button_apply.clone();

        self.tab_page.on().wm_size(move |size_info| {
            let group_box_margin = gui::dpi_x(10);
            let button_width = gui::dpi_x(75);
            let button_height = gui::dpi_y(25);
            let button_horizontal_gap = gui::dpi_x(5);
            let tab_page_client_width = size_info.client_area.cx;
            let tab_page_client_height = size_info.client_area.cy;

            let group_box_height =
                tab_page_client_height - 2 * group_box_margin - button_height - group_box_margin;

            group_box.hwnd().SetWindowPos(
                HwndPlace::None,
                POINT {
                    x: group_box_margin,
                    y: group_box_margin,
                },
                SIZE {
                    cx: tab_page_client_width - 2 * group_box_margin,
                    cy: group_box_height,
                },
                co::SWP::NOZORDER | co::SWP::NOCOPYBITS,
            )?;

            let button_vertical_position = group_box_margin + group_box_height + group_box_margin;

            let button_apply_horizontal_position =
                tab_page_client_width - group_box_margin - button_width;
            button_apply.hwnd().SetWindowPos(
                HwndPlace::None,
                POINT {
                    x: button_apply_horizontal_position,
                    y: button_vertical_position,
                },
                SIZE {
                    cx: button_width,
                    cy: button_height,
                },
                co::SWP::NOZORDER | co::SWP::NOCOPYBITS,
            )?;

            let button_select_all_toggle_horizontal_position =
                button_apply_horizontal_position - button_horizontal_gap - button_width;
            button_select_all_toggle.hwnd().SetWindowPos(
                HwndPlace::None,
                POINT {
                    x: button_select_all_toggle_horizontal_position,
                    y: button_vertical_position,
                },
                SIZE {
                    cx: button_width,
                    cy: button_height,
                },
                co::SWP::NOZORDER | co::SWP::NOCOPYBITS,
            )?;

            Ok(())
        });
    }

    fn setup_button_select_all_toggle_event(&self) {
        self.button_select_all_toggle
            .on()
            .bn_clicked(move || Ok(()));
    }

    fn setup_button_apply_event(&self) {
        self.button_apply.on().bn_clicked(move || Ok(()));
    }
}
