#![windows_subsystem = "windows"]
i18n!("locales", fallback = "en-US");

use rust_i18n::{i18n, t};
use winsafe::prelude::{GuiEventsParent, GuiWindow, Handle};

mod config;
mod error;
mod ui;

#[derive(Clone)]
pub struct MainWindow {
    main_window: winsafe::gui::WindowMain,
}

impl MainWindow {
    pub fn create_and_run() -> winsafe::AnyResult<i32> {
        rust_i18n::set_locale(&sys_locale::get_locale().unwrap_or_else(|| "en-US".to_string()));
        ui::ui_customization_hook::install_ui_customization_hook();
        error::panic_hook::install_panic_hook();

        let app_config = config::AppConfig::load();
        rust_i18n::set_locale(&app_config.language);

        let main_window = winsafe::gui::WindowMain::new(winsafe::gui::WindowMainOpts {
            title: "TOOLBOX",
            ..Default::default()
        });

        let main_window_instance = Self { main_window };
        main_window_instance.events();

        main_window_instance.main_window.run_main(None)
    }

    fn events(&self) {
        ui::menu::register_menu_events(self);

        let cloned_main_window_instance = self.clone();
        self.main_window.on().wm_create(move |_| {
            let main_window_hwnd = cloned_main_window_instance.main_window.hwnd();
            let main_menu_bar = ui::menu::build_main_menu()?;

            main_window_hwnd.SetMenu(&main_menu_bar)?;
            ui::window_utils::center_window(main_window_hwnd)?;
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
