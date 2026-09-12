//! Individual tab page implementations.
//!
//! Each submodule owns one tab page: its controls, layout, and events.
//! Pages are constructed by [`crate::ui::tab::container::TabContainer`]
//! and must implement [`From<Page> for gui::TabPage`] so they can be
//! handed to the tab control.
//!
//! # Module Structure
//! - [`settings`]             — scrollable checkbox list with apply/select-all buttons
//! - [`window_visual_styles`] — placeholder page for window visual style options

pub mod settings;
pub mod window_visual_styles;

pub use settings::SettingsPage;
pub use window_visual_styles::WindowVisualStylesPage;
