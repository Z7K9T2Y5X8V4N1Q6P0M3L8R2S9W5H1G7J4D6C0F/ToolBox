//! System visual styles and window theme manipulation.
//!
//! # Module Structure
//! - [`registry`] — registry import engine and broadcast notification
//! - [`presets`]  — visual style configuration datasets

pub mod presets;
pub mod registry;

pub use registry::apply_default_metrics;
