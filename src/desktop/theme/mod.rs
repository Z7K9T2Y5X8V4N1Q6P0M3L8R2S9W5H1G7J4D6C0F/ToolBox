//! System visual styles and window theme manipulation.
//!
//! # Module Structure
//! - [`registry`] — registry import engine using default user hive cloning

pub mod registry;

pub use registry::{
    add_extra_classic_schemes, apply_default_metrics, restore_default_classic_schemes,
};
