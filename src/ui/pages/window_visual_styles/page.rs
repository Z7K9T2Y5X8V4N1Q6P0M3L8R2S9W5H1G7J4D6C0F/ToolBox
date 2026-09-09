//! [`WindowVisualStylesPage`] — the public handle for the window visual styles tab page.
//!
//! Owns all controls and exposes only the operations the rest of the
//! application needs: construction and locale-driven text updates.

use std::{cell::RefCell, rc::Rc};

use rust_i18n::t;
use winsafe::{WString, gui, msg, prelude::*};

use super::process::ProcessManager;

/// The window visual styles tab page.
///
/// Cloning is cheap — all inner fields are reference-counted WinSafe handles or `Rc` pointers.
/// The [`From`] impl allows this to be handed directly to [`gui::Tab`] pages.
#[derive(Clone)]
pub struct WindowVisualStylesPage {
    tab_page: gui::TabPage,
    edit: gui::Edit,
    listview: gui::ListView,
    process_manager: Rc<RefCell<ProcessManager>>,
}

impl From<WindowVisualStylesPage> for gui::TabPage {
    fn from(window_visual_styles_page: WindowVisualStylesPage) -> gui::TabPage {
        window_visual_styles_page.tab_page.clone()
    }
}

impl WindowVisualStylesPage {
    /// Construct the window visual styles page, creating all controls and registering all events.
    ///
    /// Must be called before the message loop starts, on the same thread as
    /// the parent window.
    pub fn new(parent_window: &(impl GuiParent + 'static), status_bar: gui::StatusBar) -> Self {
        let process_manager = Rc::new(RefCell::new(ProcessManager::new()));
        let tab_page = super::build::create_tab_page(parent_window);
        let edit = super::build::create_edit(&tab_page);
        let listview = super::build::create_listview(&tab_page);

        super::event::setup_all_events(&tab_page, &edit, &listview, &process_manager, &status_bar);

        Self {
            tab_page,
            edit,
            listview,
            process_manager,
        }
    }

    /// Re-translate all visible text labels to the current locale.
    ///
    /// Updates the Edit cue banner, the ListView column headers, and re-applies the
    /// active sort indicator arrow to the header.
    pub fn update_texts(&self) -> winsafe::AnyResult<()> {
        // 1. Update Edit cue banner
        unsafe {
            self.edit
                .hwnd()
                .SendMessage(msg::EmSetCueBanner {
                    show_even_with_focus: false,
                    text: WString::from_str(t!("COMBOBOX_CUE_BANNER")),
                })
                .ok();
        }

        // 2. Update ListView column headers
        self.listview
            .cols()
            .get(0)
            .set_title(&t!("LISTVIEW_COLUMN_PROCESS_NAME"))?;

        self.listview
            .cols()
            .get(1)
            .set_title(&t!("LISTVIEW_COLUMN_PID"))?;

        Ok(())
    }
}
