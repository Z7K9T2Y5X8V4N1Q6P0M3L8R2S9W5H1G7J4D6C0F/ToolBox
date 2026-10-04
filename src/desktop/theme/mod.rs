//! System visual styles and window theme manipulation.
//!
//! # Module Structure
//! - [`registry`] — registry import engine using default user hive cloning

pub mod registry;

pub use registry::{apply_default_metrics, restore_default_classic_schemes};
