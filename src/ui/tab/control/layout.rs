use super::rect_calculator;
use winsafe::{HwndPlace, POINT, SIZE, co, gui, prelude::*};

const TAB_CONTROL_MARGIN: i32 = 10;

pub(super) fn resize_tab_control(
    tab_control: &gui::Tab,
    window_client_width: i32,
    window_client_height: i32,
) -> winsafe::AnyResult<()> {
    let tab_control_margin = gui::dpi_x(TAB_CONTROL_MARGIN);
    let tab_control_size = SIZE {
        cx: window_client_width - (tab_control_margin * 2),
        cy: window_client_height - (tab_control_margin * 2),
    };

    tab_control.hwnd().SetWindowPos(
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

pub(super) fn resize_current_tab_page(
    tab_control: &gui::Tab,
    tab_pages: &[gui::TabPage],
) -> winsafe::AnyResult<()> {
    let Some(selected_tab_control_item) = tab_control.items().selected() else {
        return Ok(());
    };

    let Some(target_tab_page) = tab_pages.get(selected_tab_control_item.index() as usize) else {
        return Ok(());
    };

    let tab_page_screen_to_client_rect =
        rect_calculator::calculate_tab_page_rect(tab_control.hwnd())?;

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
