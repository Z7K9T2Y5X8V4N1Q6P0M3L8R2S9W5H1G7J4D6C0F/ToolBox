#![windows_subsystem = "windows"]

#[derive(Clone)]
pub struct MainWindow {
    main_window: winsafe::gui::WindowMain,
}

impl MainWindow {
    pub fn create_and_run() -> winsafe::AnyResult<i32> {
        let main_window = winsafe::gui::WindowMain::new(winsafe::gui::WindowMainOpts {
            title: "TOOLBOX",
            size: winsafe::gui::dpi(300, 150),
            ..Default::default()
        });

        let main_window_instance = Self { main_window };
        main_window_instance.events();

        main_window_instance.main_window.run_main(None)
    }

    fn events(&self) {}
}

fn main() {
    if let Err(error) = MainWindow::create_and_run() {
        eprintln!("{}", error);
    }
}
