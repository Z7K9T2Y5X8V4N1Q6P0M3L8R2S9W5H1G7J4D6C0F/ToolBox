//! Application-level error handling.
//!
//! # Module Structure
//! - [`dialog`] — displays global modal error dialogs without a parent window
//! - [`panic`]  — installs a custom panic hook that shows a Win32 error dialog
//!   instead of printing to stderr (which is invisible in a GUI application)

mod dialog;
pub mod panic;

pub use dialog::show_error_dialog;
pub use panic::install_panic_hook;
