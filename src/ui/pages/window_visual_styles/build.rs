//! Window visual styles page control construction.
//!
//! Each function creates exactly one control. All controls are created during
//! [`WindowVisualStylesPage::new`] before the message loop starts — Win32 requires
//! controls to be created on the same thread as their parent window.

use rust_i18n::t;
use winsafe::{co, gui, prelude::*};

/// Create the [`gui::TabPage`] that hosts all window visual styles controls.
pub(super) fn create_tab_page(parent_window: &(impl GuiParent + 'static)) -> gui::TabPage {
    gui::TabPage::new(
        parent_window,
        gui::TabPageOpts {
            class_style: co::CS::HREDRAW | co::CS::VREDRAW,
            ..Default::default()
        },
    )
}

/// Create a single-line Edit control for user input.
///
/// `ES_AUTOHSCROLL` enables automatic horizontal scrolling when the text
/// exceeds the width of the edit control.
pub(super) fn create_edit(parent_window: &gui::TabPage) -> gui::Edit {
    gui::Edit::new(
        parent_window,
        gui::EditOpts {
            window_ex_style: co::WS_EX::CLIENTEDGE,
            control_style: co::ES::AUTOHSCROLL,
            ..Default::default()
        },
    )
}

/// Create a ListView control with report view style and predefined columns.
///
/// `LVS_REPORT` creates a multi-column list view with column headers.
/// `LVS_SINGLESEL` restricts selection to a single item at a time.
/// `LVS_EX::FULLROWSELECT` highlights the entire row when an item is selected,
/// rather than just the first column. `LVS_EX::GRIDLINES` displays grid lines
/// between rows and columns for better visual separation.
pub(super) fn create_listview(parent_window: &gui::TabPage) -> gui::ListView {
    gui::ListView::new(
        parent_window,
        gui::ListViewOpts {
            control_style: co::LVS::REPORT | co::LVS::SINGLESEL,
            control_ex_style: co::LVS_EX::FULLROWSELECT
                | co::LVS_EX::GRIDLINES
                | co::LVS_EX::DOUBLEBUFFER,
            columns: &[
                (&t!("LISTVIEW_COLUMN_PROCESS_NAME"), 0),
                (&t!("LISTVIEW_COLUMN_PID"), 0),
            ],
            ..Default::default()
        },
    )
}
