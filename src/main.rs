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
        Self::initialize_application();

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
        main_window_instance.register_events()?;

        main_window_instance.main_window.run_main(None)
    }

    fn initialize_application() {
        Self::setup_initial_locale();
        ui::ui_customization_hook::install_ui_customization_hook();
        error::panic_hook::install_panic_hook();
        Self::setup_config_locale();
    }

    fn setup_initial_locale() {
        let system_locale = sys_locale::get_locale()
            .unwrap_or_else(|| AppLanguage::EnUs.as_locale_str().to_string());
        rust_i18n::set_locale(&system_locale);
    }

    fn setup_config_locale() {
        let app_config = config::AppConfig::load();
        rust_i18n::set_locale(&app_config.language.as_locale_str());
    }

    fn register_events(&self) -> winsafe::AnyResult<()> {
        ui::menu::register_menu_events(self);
        self.register_window_create_event();
        self.register_window_min_max_info_event();
        self.register_window_size_event();
        self.register_window_app_message_event();
        Ok(())
    }

    fn register_window_create_event(&self) {
        let cloned_main_window_instance_for_window_create_event = self.clone();
        self.main_window.on().wm_create(move |_| {
            let main_window_hwnd = cloned_main_window_instance_for_window_create_event
                .main_window
                .hwnd();
            let main_menu_bar = ui::menu::build_main_menu()?;
            main_window_hwnd.SetMenu(&main_menu_bar)?;
            ui::window_utils::center_and_resize_window(main_window_hwnd)?;
            Ok(0)
        });
    }

    fn register_window_min_max_info_event(&self) {
        self.main_window.on().wm_get_min_max_info(|min_max| {
            ui::window_utils::apply_minimize_window_size(min_max.info);
            Ok(())
        });
    }

    fn register_window_size_event(&self) {
        let cloned_main_window_instance_for_window_size_event = self.clone();
        self.main_window.on().wm_size(move |size_info| {
            cloned_main_window_instance_for_window_size_event
                .tab_pages
                .resize(size_info.client_area.cx, size_info.client_area.cy)?;
            Ok(())
        });
    }

    fn register_window_app_message_event(&self) {
        let cloned_main_window_instance_for_window_app_message_event = self.clone();
        self.main_window.on().wm(winsafe::co::WM::APP, move |_| {
            if let Some(error_message) = cloned_main_window_instance_for_window_app_message_event
                .pending_error_message
                .borrow_mut()
                .take()
            {
                cloned_main_window_instance_for_window_app_message_event
                    .main_window
                    .hwnd()
                    .MessageBox(
                        &error_message,
                        &rust_i18n::t!("ERROR"),
                        winsafe::co::MB::OK | winsafe::co::MB::ICONWARNING,
                    )?;
            }
            Ok(0)
        });
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
