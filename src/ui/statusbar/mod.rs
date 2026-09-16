//! Status bar construction and hover event handling.
//!
//! The status bar sits at the bottom of the main window and displays
//! a description of whichever checkbox the mouse is currently hovering over.
//!
//! # Module Structure
//! - [`build`] — creates the [`winsafe::gui::StatusBar`] control
//! - [`event`] — registers mouse hover/leave events on each checkbox

mod build;
mod event;

pub use build::create_status_bar;
pub use event::register_hover_events;
