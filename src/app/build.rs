//! Main window construction and message loop entry point.

use std::{cell::RefCell, rc::Rc};

use rust_i18n::t;
use winsafe::{AnyResult, co, gui, msg, prelude::*};

use super::event;
use crate::ui::{font::FontManager, statusbar, tab::TabContainer};

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
        let window_title = t!("TOOLBOX_TITLE");
        let main_window = gui::WindowMain::new(gui::WindowMainOpts {
            title: &window_title,
            class_icon: gui::Icon::Id(1),
            style: co::WS::OVERLAPPEDWINDOW,
            ..Default::default()
        });

        let status_bar = statusbar::create_status_bar(&main_window);
        let font_manager = Rc::new(RefCell::new(FontManager::new()));
        let tab_container =
            TabContainer::new(&main_window, status_bar.clone(), font_manager.clone());

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

    /// Enqueue an error message to be displayed asynchronously on the UI thread.
    ///
    /// Posts a [`winsafe::co::WM::APP`] message to the root window message queue.
    /// This prevents modal dialogs from blocking inner menu dispatch loops or
    /// causing nested message pump reentrancy.
    pub fn post_deferred_error(&self, error_message: String) {
        self.pending_error_message.replace(Some(error_message));

        unsafe {
            self.main_window
                .hwnd()
                .PostMessage(msg::Wm {
                    msg_id: co::WM::APP,
                    wparam: 0,
                    lparam: 0,
                })
                .ok();
        }
    }
}
