//! Main window event handlers.
//!
//! All Win32 window messages for [`MainWindow`] are registered here.
//! UI component events (menu, tab, etc.) are registered in their own modules
//! and called from [`register_all_events`].

use rust_i18n::t;
use windows::Win32::UI::WindowsAndMessaging::WM_SETTINGCHANGE;
use winsafe::gui;
use winsafe::prelude::{GuiEventsParent, GuiEventsWindow, GuiWindow};

use super::build::MainWindow;
use crate::ui;

/// Register all event handlers for the main window.
///
/// Must be called once after [`MainWindow`] is constructed and before
/// the message loop starts.
pub fn register_all_events(main_window_instance: &MainWindow) -> winsafe::AnyResult<()> {
    ui::menu::register_menu_events(main_window_instance);
    register_window_create_event(main_window_instance);
    register_window_min_max_info_event(main_window_instance);
    register_window_size_event(main_window_instance);
    register_window_app_message_event(main_window_instance);
    register_window_setting_change_event(main_window_instance);
    Ok(())
}

/// On WM_CREATE: attach the menu bar, center window, and apply the initial system font.
fn register_window_create_event(main_window_instance: &MainWindow) {
    let cloned_main_window_instance = main_window_instance.clone();
    main_window_instance.main_window.on().wm_create(move |_| {
        let main_window_hwnd = cloned_main_window_instance.main_window.hwnd();
        let main_menu_bar = ui::menu::build_main_menu()?;
        main_window_hwnd.SetMenu(&main_menu_bar)?;
        ui::window::layout::center_and_resize_window(main_window_hwnd)?;

        cloned_main_window_instance
            .font_manager
            .borrow_mut()
            .sync_system_font(main_window_hwnd)?;

        Ok(0)
    });
}

/// On WM_GETMINMAXINFO: enforce a minimum window size so the UI remains usable.
fn register_window_min_max_info_event(main_window_instance: &MainWindow) {
    main_window_instance
        .main_window
        .on()
        .wm_get_min_max_info(|min_max| {
            ui::window::layout::apply_minimum_window_size(min_max.info);
            Ok(())
        });
}

/// On WM_SIZE: resize the tab container to fill the available client area
/// below the status bar.
fn register_window_size_event(main_window_instance: &MainWindow) {
    let cloned_main_window_instance = main_window_instance.clone();
    main_window_instance
        .main_window
        .on()
        .wm_size(move |size_info| {
            relayout_main_window_contents(
                &cloned_main_window_instance.tab_container,
                &cloned_main_window_instance.status_bar,
                size_info.client_area.cx,
                size_info.client_area.cy,
            )?;
            Ok(())
        });
}

/// Recalculate main window content dimensions and resize child containers.
pub(crate) fn relayout_main_window_contents(
    tab_container: &ui::tab::container::TabContainer,
    status_bar: &gui::StatusBar,
    client_width: i32,
    client_height: i32,
) -> winsafe::AnyResult<()> {
    let status_bar_height = status_bar
        .hwnd()
        .GetWindowRect()
        .map(|rect| rect.bottom - rect.top)
        .unwrap_or(0);

    let available_height_for_tab = client_height - status_bar_height;
    tab_container.resize(client_width, available_height_for_tab)?;
    Ok(())
}

/// On WM_SETTINGCHANGE: detect system font/metric updates and apply them dynamically.
fn register_window_setting_change_event(main_window_instance: &MainWindow) {
    let cloned_main_window_instance = main_window_instance.clone();
    main_window_instance.main_window.on().wm(
        unsafe { winsafe::co::WM::from_raw(WM_SETTINGCHANGE) },
        move |_| {
            let main_window_hwnd = cloned_main_window_instance.main_window.hwnd();

            let font_sync_result = cloned_main_window_instance
                .font_manager
                .borrow_mut()
                .sync_system_font(main_window_hwnd)?;

            if font_sync_result == crate::ui::font::FontSyncResult::Changed {
                cloned_main_window_instance
                    .tab_container
                    .handle_font_changed()?;

                let main_window_client_rect = main_window_hwnd.GetClientRect()?;
                let main_window_client_width =
                    main_window_client_rect.right - main_window_client_rect.left;
                let main_window_client_height =
                    main_window_client_rect.bottom - main_window_client_rect.top;

                relayout_main_window_contents(
                    &cloned_main_window_instance.tab_container,
                    &cloned_main_window_instance.status_bar,
                    main_window_client_width,
                    main_window_client_height,
                )?;
            }

            Ok(0)
        },
    );
}

/// On WM_APP: display a deferred error message dialog.
fn register_window_app_message_event(main_window_instance: &MainWindow) {
    let cloned_main_window_instance = main_window_instance.clone();
    main_window_instance
        .main_window
        .on()
        .wm(winsafe::co::WM::APP, move |_| {
            if let Some(error_message) = cloned_main_window_instance
                .pending_error_message
                .borrow_mut()
                .take()
            {
                cloned_main_window_instance.main_window.hwnd().MessageBox(
                    &error_message,
                    &t!("ERROR"),
                    winsafe::co::MB::OK | winsafe::co::MB::ICONWARNING,
                )?;
            }
            Ok(0)
        });
}
