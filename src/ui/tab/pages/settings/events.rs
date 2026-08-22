use winsafe::{gui, prelude::*};

use crate::ui;

use super::layout::SettingsPageLayout;

pub(super) fn setup_all_events(
    tab_page: &gui::TabPage,
    group_box: &gui::Button,
    button_select_all_toggle: &gui::Button,
    button_apply: &gui::Button,
) {
    setup_tab_page_background_event(tab_page);
    setup_resize_event(tab_page, group_box, button_select_all_toggle, button_apply);
    setup_button_select_all_toggle_event(button_select_all_toggle);
    setup_button_apply_event(button_apply);
}

fn setup_tab_page_background_event(tab_page: &gui::TabPage) {
    ui::tab::utils::setup_tab_page_background_events(tab_page);
}

fn setup_resize_event(
    tab_page: &gui::TabPage,
    group_box: &gui::Button,
    button_select_all_toggle: &gui::Button,
    button_apply: &gui::Button,
) {
    let cloned_group_box_for_window_size_event = group_box.clone();
    let cloned_button_select_all_toggle_for_window_size_event = button_select_all_toggle.clone();
    let cloned_button_apply_for_window_size_event = button_apply.clone();
    tab_page.on().wm_size(move |size_info| {
        let layout =
            SettingsPageLayout::calculate(size_info.client_area.cx, size_info.client_area.cy);

        ui::tab::utils::reposition_control(
            cloned_group_box_for_window_size_event.hwnd(),
            layout.group_box_position,
            layout.group_box_size,
        )?;

        ui::tab::utils::reposition_control(
            cloned_button_apply_for_window_size_event.hwnd(),
            layout.button_apply_position,
            layout.button_apply_size,
        )?;

        ui::tab::utils::reposition_control(
            cloned_button_select_all_toggle_for_window_size_event.hwnd(),
            layout.button_select_all_toggle_position,
            layout.button_select_all_toggle_size,
        )?;

        Ok(())
    });
}

fn setup_button_select_all_toggle_event(button_select_all_toggle: &gui::Button) {
    button_select_all_toggle.on().bn_clicked(move || Ok(()));
}

fn setup_button_apply_event(button_apply: &gui::Button) {
    button_apply.on().bn_clicked(move || Ok(()));
}
