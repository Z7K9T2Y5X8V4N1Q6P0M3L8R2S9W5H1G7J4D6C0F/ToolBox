pub fn scale_by_dpi(value: i32, dpi: i32) -> i32 {
    (value * dpi) / 96
}

pub fn center_and_resize_window(
    main_window_handle: &winsafe::HWND,
    window_dpi: i32,
) -> winsafe::AnyResult<()> {
    let calculated_window_width = scale_by_dpi(310, window_dpi);
    let calculated_window_height = scale_by_dpi(530, window_dpi);

    let system_screen_width = winsafe::GetSystemMetrics(winsafe::co::SM::CXSCREEN);
    let system_screen_height = winsafe::GetSystemMetrics(winsafe::co::SM::CYSCREEN);
    let centered_window_position_x = (system_screen_width - calculated_window_width) / 2;
    let centered_window_position_y = (system_screen_height - calculated_window_height) / 2;

    main_window_handle.SetWindowPos(
        winsafe::HwndPlace::None,
        winsafe::POINT {
            x: centered_window_position_x,
            y: centered_window_position_y,
        },
        winsafe::SIZE {
            cx: calculated_window_width,
            cy: calculated_window_height,
        },
        winsafe::co::SWP::NOZORDER,
    )?;

    Ok(())
}

pub fn apply_minimize_window_size(min_max_info: &mut winsafe::MINMAXINFO, window_dpi: i32) {
    min_max_info.ptMinTrackSize.x = scale_by_dpi(310, window_dpi);
    min_max_info.ptMinTrackSize.y = scale_by_dpi(530, window_dpi);
}
