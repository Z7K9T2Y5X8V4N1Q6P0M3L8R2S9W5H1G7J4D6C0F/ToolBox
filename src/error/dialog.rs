//! Global modal error dialog utilities.
//!
//! Provides top-level message boxes displayed using the desktop window handle
//! (`HWND::NULL`), suitable for fatal initialization errors or background crashes
//! before or outside the main window lifecycle.

use rust_i18n::t;
use winsafe::{HWND, co, prelude::Handle};

/// Display a modal error dialog using the top-level desktop window.
///
/// Uses [`HWND::NULL`] as the parent handle so the dialog can be displayed
/// even when no main window exists yet.
pub fn show_error_dialog(message: &str) {
    HWND::NULL
        .MessageBox(message, &t!("ERROR"), co::MB::OK | co::MB::ICONERROR)
        .ok();
}
