//! Main window construction and message loop entry point.

use std::{cell::RefCell, rc::Rc};

use rust_i18n::t;
use winsafe::{AnyResult, co, gui};

use crate::ui::{font::FontManager, statusbar, tab::TabContainer};

use super::{event, init};

/// The root window of the application.
///
/// Holds references to all top-level UI components. Cloning is cheap
/// because all inner fields use `Rc` or WinSafe's own reference-counted handles.
///
/// The `pending_error_message` field is a deferred error channel: operations
/// that cannot show a modal dialog synchronously (e.g. inside a menu handler)
/// store their error here and post [`winsafe::co::WM::APP`] to trigger display
/// after the current message processing completes.
#[derive(Clone)]
pub struct MainWindow {
    pub(crate) main_window: gui::WindowMain,
    pub(crate) pending_error_message: Rc<RefCell<Option<String>>>,
    pub(crate) tab_container: TabContainer,
    pub(crate) status_bar: gui::StatusBar,
    pub(crate) font_manager: Rc<RefCell<FontManager>>,
}

impl MainWindow {
    /// Create the main window, initialize all UI components, and run the message loop.
    ///
    /// Returns when the user closes the window. The return value is the exit
    /// code that should be passed back to the OS.
    pub fn create_and_run() -> AnyResult<i32> {
        init::initialize_application();

        let window_title = t!("TOOLBOX_TITLE");
        let main_window = gui::WindowMain::new(gui::WindowMainOpts {
            title: &window_title,
            style: co::WS::OVERLAPPEDWINDOW,
            ..Default::default()
        });

        let status_bar = statusbar::create_status_bar(&main_window);
        let tab_container = TabContainer::new(&main_window, status_bar.clone());
        let font_manager = Rc::new(RefCell::new(FontManager::new()));

        let main_window_instance = Self {
            main_window,
            pending_error_message: Rc::new(RefCell::new(None)),
            tab_container,
            status_bar,
            font_manager,
        };

        event::register_all_events(&main_window_instance)?;
        main_window_instance.main_window.run_main(None)
    }
}
