//! System management and core OS interaction routines.
//!
//! # Module Structure
//! - [`process`] — system process operations, including desktop shell restarts

pub mod process;

pub use process::restart_desktop_shell;
