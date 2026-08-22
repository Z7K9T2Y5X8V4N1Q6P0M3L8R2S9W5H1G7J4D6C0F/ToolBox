use winsafe::{HWND, RECT, msg};

pub(super) fn calculate_tab_page_rect(tab_control_hwnd: &HWND) -> winsafe::AnyResult<RECT> {
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
