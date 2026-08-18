use winsafe::gui;

const WINDOW_WIDTH: i32 = 310;
const WINDOW_HEIGHT: i32 = 530;

pub fn center_and_resize_window(main_window_handle: &winsafe::HWND) -> winsafe::AnyResult<()> {
    let calculated_window_size = winsafe::SIZE {
        cx: gui::dpi_x(WINDOW_WIDTH),
        cy: gui::dpi_y(WINDOW_HEIGHT),
    };

    let system_screen_width = winsafe::GetSystemMetrics(winsafe::co::SM::CXSCREEN);
    let system_screen_height = winsafe::GetSystemMetrics(winsafe::co::SM::CYSCREEN);
    let centered_window_position = winsafe::POINT {
        x: (system_screen_width - calculated_window_size.cx) / 2,
        y: (system_screen_height - calculated_window_size.cy) / 2,
    };

    main_window_handle.SetWindowPos(
        winsafe::HwndPlace::None,
        centered_window_position,
        calculated_window_size,
        winsafe::co::SWP::NOZORDER,
    )?;

    Ok(())
}

pub fn apply_minimize_window_size(min_max_info: &mut winsafe::MINMAXINFO) {
    min_max_info.ptMinTrackSize = winsafe::POINT {
        x: gui::dpi_x(WINDOW_WIDTH),
        y: gui::dpi_y(WINDOW_HEIGHT),
    };
}
