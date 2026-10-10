//! System visual styles, metric restoration, and appearance schemes manipulation.
//!
//! # Module Structure
//! - [`basic`]    — global Windows Basic visual style monitoring engine
//! - [`classic`]  — global Windows Classic visual style switching engine
//! - [`hive`]     — offline default user profile hive path resolution and RAII mounting guard
//! - [`registry`] — registry cloning engine and classic metrics/schemes restoration logic
//! - [`schemes`]  — predefined classic desktop appearance binary payloads

pub mod basic;
pub mod classic;
pub mod hive;
pub mod registry;
pub mod schemes;

pub use basic::{GlobalBasicStylesState, GlobalBasicStylesWatcher};
pub use classic::{GlobalClassicStylesState, GlobalClassicStylesWatcher};
pub use registry::{
    add_extra_classic_schemes, apply_default_metrics, restore_default_classic_schemes,
};
