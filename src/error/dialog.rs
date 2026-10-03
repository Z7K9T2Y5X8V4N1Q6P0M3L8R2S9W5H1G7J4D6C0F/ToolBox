//! Global modal error dialog utilities.
//!
//! Provides top-level message boxes displayed using the desktop window handle
//! (`HWND::NULL`), suitable for fatal initialization errors or background crashes
//! before or outside the main window lifecycle.

use rust_i18n::t;
use winsafe::{HWND, co, prelude::Handle};

/// Display an ownerless, blocking error dialog.
///
/// Uses [`HWND::NULL`] as the parent handle so the dialog can be displayed
/// even when no main window exists yet.
pub fn show_error_dialog(error_message: &str) {
    HWND::NULL
        .MessageBox(
            error_message,
            &t!("ERROR"),
            co::MB::OK | co::MB::ICONERROR | co::MB::TASKMODAL,
        )
        .ok();
}

/// Display an ownerless, blocking fatal error dialog.
///
/// Uses [`HWND::NULL`] as the parent handle so the dialog can be displayed
/// during panics or critical failures even when the main window does not exist.
pub fn show_fatal_error_dialog(fatal_error_message: &str) {
    HWND::NULL
        .MessageBox(
            fatal_error_message,
            &t!("FATAL_ERROR_TITLE"),
            co::MB::OK | co::MB::ICONERROR | co::MB::TASKMODAL,
        )
        .ok();
}
