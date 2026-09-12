//! Application-level error handling.
//!
//! # Module Structure
//! - [`panic`] — installs a custom panic hook that shows a Win32 error dialog
//!   instead of printing to stderr (which is invisible in a GUI application)
pub mod panic;

pub use panic::install_panic_hook;
