//! Desktop management, shell manipulation, and environment customization.
//!
//! # Module Structure
//! - [`session`] — active desktop session user resolution and registry hive access
//! - [`shell`]   — Windows desktop shell process operations (e.g. Explorer restarts)
//! - [`theme`]   — desktop visual styles, colors, and window metric customization

pub mod session;
pub mod shell;
pub mod theme;
