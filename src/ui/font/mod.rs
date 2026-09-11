//! System UI font management and dynamic synchronization.
//!
//! # Module Structure
//! - [`manager`] — [`FontManager`] struct for holding and applying the system font

mod manager;

pub use manager::{FontManager, FontSyncResult};
