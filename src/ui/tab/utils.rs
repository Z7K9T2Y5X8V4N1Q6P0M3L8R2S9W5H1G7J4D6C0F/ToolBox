use winsafe::{HBRUSH, HWND, HwndPlace, POINT, SIZE, co, gui, prelude::*};

pub(super) fn setup_tab_page_background_events(tab_page: &gui::TabPage) {
    let cloned_tab_page_for_background = tab_page.clone();
    tab_page.on().wm_erase_bkgnd(move |erase_bkgnd_params| {
        let tab_page_content_client_rect = cloned_tab_page_for_background.hwnd().GetClientRect()?;
        let background_brush = HBRUSH::GetSysColorBrush(co::COLOR::BTNFACE)?;
        erase_bkgnd_params
            .hdc
            .FillRect(tab_page_content_client_rect, &background_brush)?;
        Ok(1)
    });
}

pub(super) fn reposition_control(
    control_hwnd: &HWND,
    position: POINT,
    size: SIZE,
) -> winsafe::AnyResult<()> {
    control_hwnd.SetWindowPos(
        HwndPlace::None,
        position,
        size,
        co::SWP::NOZORDER | co::SWP::NOCOPYBITS,
    )?;
    Ok(())
}
