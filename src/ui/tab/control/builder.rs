use rust_i18n::t;
use winsafe::{gui, prelude::*};

use crate::ui::tab::pages::{settings::SettingsPage, window_visual_styles::WindowVisualStylesPage};

pub(super) fn create_tab_control(
    parent_window: &(impl GuiParent + 'static),
    settings_page: &SettingsPage,
    window_visual_styles_page: &WindowVisualStylesPage,
) -> gui::Tab {
    let tab_control_titles = get_tab_control_titles();

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

pub(super) fn get_tab_control_titles() -> Vec<String> {
    vec![
        t!("TAB_SETTINGS").to_string(),
        t!("TAB_WINDOW_VISUAL_STYLES").to_string(),
    ]
}
