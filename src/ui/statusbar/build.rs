//! Status bar control construction.

use winsafe::{gui, prelude::GuiParent};

/// Create a single-part proportional status bar attached to the given parent window.
///
/// The status bar is automatically repositioned by Win32 when the parent
/// window is resized, so no manual resize handling is needed.
pub fn create_status_bar(parent_window: &(impl GuiParent + 'static)) -> gui::StatusBar {
    gui::StatusBar::new(parent_window, &[gui::SbPart::Proportional(1)])
}
