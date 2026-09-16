//! Settings tab page.
//!
//! Presents a scrollable list of checkboxes for toggling Windows system
//! settings, along with "Select All / Deselect All" and "Apply" buttons.
//!
//! # Layout
//! ```text
//! ┌─ group box ──────────────────────────────┐
//! │ ┌─ scrollable panel ──────────────────┐  │
//! │ │  ☐ Disable Windows Update           │  │
//! │ │  ☐ Pause Windows Update             │  │
//! │ │  ...                                │▓ │
//! │ └─────────────────────────────────────┘  │
//! └──────────────────────────────────────────┘
//!                        [Select All]  [Apply]
//! ```
//!
//! # Module Structure
//! - [`state`]  — [`CheckboxId`] enum: labels, descriptions, and control IDs
//! - [`build`]  — constructs all controls (tab page, group box, panels, checkboxes, buttons)
//! - [`layout`] — DPI-aware layout constants, [`layout::CheckboxLayoutCalculator`], [`layout::SettingsPageLayout`]
//! - [`scroll`] — vertical scrollbar and mouse wheel event handlers
//! - [`event`]  — resize, button, and status bar hover event registration
//! - [`page`]   — [`SettingsPage`] struct: public API for construction and text updates

mod build;
mod event;
mod layout;
mod page;
mod scroll;
pub mod state;

pub use page::SettingsPage;
pub use state::CheckboxId;
