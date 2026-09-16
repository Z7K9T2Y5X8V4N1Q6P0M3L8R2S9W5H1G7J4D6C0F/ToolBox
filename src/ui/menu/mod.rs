//! Main menu bar construction and event handling.
//!
//! The menu bar contains two top-level submenus:
//! - **Options** — application actions (e.g. restart Explorer)
//! - **Language** — locale switching, with the active language grayed out
//!
//! # Module Structure
//! - [`state`] — menu command ID constants
//! - [`build`] — constructs and rebuilds the [`winsafe::HMENU`] tree
//! - [`event`] — registers WM_COMMAND handlers on [`crate::app::MainWindow`]

mod build;
mod event;
pub mod state;

pub use build::build_main_menu;
pub use event::register_menu_events;
