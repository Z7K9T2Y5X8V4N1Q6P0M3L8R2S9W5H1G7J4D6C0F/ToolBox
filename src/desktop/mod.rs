//! Desktop management, shell manipulation, and environment customization.
//!
//! # Module Structure
//! - [`shell`]   — Windows desktop shell process operations (e.g. Explorer restarts)
//! - [`session`] — active desktop session user resolution and registry hive access
//! - [`theme`]   — desktop visual styles, colors, and window metric customization

pub mod session;
pub mod shell;
pub mod theme;

pub use shell::restart_desktop_shell;
