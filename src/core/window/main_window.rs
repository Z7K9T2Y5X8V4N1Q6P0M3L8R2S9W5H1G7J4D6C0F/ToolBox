use std::{cell::RefCell, rc::Rc};

use rust_i18n::t;

use crate::{
    core::{events, init},
    ui,
};

#[derive(Clone)]
pub struct MainWindow {
    pub(crate) main_window: winsafe::gui::WindowMain,
    pub(crate) pending_error_message: Rc<RefCell<Option<String>>>,
    pub(crate) tab_pages: ui::tab::tab_pages::TabPages,
}

impl MainWindow {
    pub fn create_and_run() -> winsafe::AnyResult<i32> {
        init::initialize_application();

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

        events::register_all_events(&main_window_instance)?;
        main_window_instance.main_window.run_main(None)
    }
}
