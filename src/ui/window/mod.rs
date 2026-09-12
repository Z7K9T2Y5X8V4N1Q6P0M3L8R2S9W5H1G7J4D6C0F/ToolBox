//! Main window utilities: UI customization hook and window layout helpers.
//!
//! # Module Structure
//! - [`hook`]   — installs a `WH_CALLWNDPROC` hook that strips Win32 visual
//!                themes and disables DWM transition animations for every window
//!                created on this thread
//! - [`layout`] — centers the main window on screen and enforces a minimum size

pub mod hook;
pub mod layout;

pub use hook::install_ui_customization_hook;
