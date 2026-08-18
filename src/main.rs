#![windows_subsystem = "windows"]
i18n!("locales", fallback = "en-US");

use std::{cell::RefCell, rc::Rc};

use rust_i18n::{i18n, t};
use winsafe::prelude::{GuiEventsParent, GuiEventsWindow, GuiWindow, Handle};

use crate::config::AppLanguage;

mod config;
mod error;
mod ui;

#[derive(Clone)]
pub struct MainWindow {
    main_window: winsafe::gui::WindowMain,
    pending_error_message: Rc<RefCell<Option<String>>>,
    tab_pages: ui::tab::tab_pages::TabPages,
}

impl MainWindow {
    pub fn create_and_run() -> winsafe::AnyResult<i32> {
        rust_i18n::set_locale(
            &sys_locale::get_locale()
                .unwrap_or_else(|| AppLanguage::EnUs.as_locale_str().to_string()),
        );
        ui::ui_customization_hook::install_ui_customization_hook();
        error::panic_hook::install_panic_hook();

        let app_config = config::AppConfig::load();
        rust_i18n::set_locale(&app_config.language.as_locale_str());

        let window_title = t!("TOOLBOX_TITLE");
        let main_window = winsafe::gui::WindowMain::new(winsafe::gui::WindowMainOpts {
            title: &window_title,
            style: winsafe::co::WS::OVERLAPPEDWINDOW,
            ..Default::default()
        });

        let tab_pages = ui::tab::tab_pages::TabPages::new(&main_window);

        let main_window_instance = Self {
            main_window,
            pending_error_message: Rc::new(RefCell::new(None)),
            tab_pages,
        };
        main_window_instance.events()?;

        main_window_instance.main_window.run_main(None)
    }

    fn events(&self) -> winsafe::AnyResult<()> {
        ui::menu::register_menu_events(self);

        let cloned_main_window_instance = self.clone();
        self.main_window.on().wm_create(move |_| {
            let main_window_hwnd = cloned_main_window_instance.main_window.hwnd();

            let main_menu_bar = ui::menu::build_main_menu()?;

            main_window_hwnd.SetMenu(&main_menu_bar)?;
            ui::window_utils::center_and_resize_window(main_window_hwnd)?;

            Ok(0)
        });

        self.main_window.on().wm_get_min_max_info(move |min_max| {
            ui::window_utils::apply_minimize_window_size(min_max.info);
            Ok(())
        });

        let cloned_main_window_instance_for_size = self.clone();
        self.main_window.on().wm_size(move |size_info| {
            let client_width = size_info.client_area.cx;
            let client_height = size_info.client_area.cy;
            cloned_main_window_instance_for_size
                .tab_pages
                .resize(client_width, client_height)?;
            Ok(())
        });

        let cloned_main_window_instance_for_app_message = self.clone();
        self.main_window.on().wm(winsafe::co::WM::APP, move |_| {
            let main_window_hwnd = cloned_main_window_instance_for_app_message
                .main_window
                .hwnd();

            if let Some(error_message) = cloned_main_window_instance_for_app_message
                .pending_error_message
                .borrow_mut()
                .take()
            {
                main_window_hwnd.MessageBox(
                    &error_message,
                    &rust_i18n::t!("ERROR"),
                    winsafe::co::MB::OK | winsafe::co::MB::ICONWARNING,
                )?;
            }
            Ok(0)
        });

        Ok(())
    }
}

fn main() {
    if let Err(error) = MainWindow::create_and_run() {
        winsafe::HWND::NULL
            .MessageBox(
                &error.to_string(),
                &t!("ERROR"),
                winsafe::co::MB::OK | winsafe::co::MB::ICONERROR,
            )
            .ok();
    }
}
