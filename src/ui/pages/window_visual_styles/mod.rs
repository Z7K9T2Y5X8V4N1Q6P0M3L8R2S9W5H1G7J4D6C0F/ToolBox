//! Window visual styles tab page.
//!
//! Presents a single-line edit control for user input and a ListView control
//! for displaying and managing system processes in real-time.
//!
//! # Layout
//! ```text
//! ┌─ tab page ───────────────────────────────┐
//! │  ┌─ edit ─────────────────────────────┐  │
//! │  │ [input field                     ] │  │
//! │  └────────────────────────────────────┘  │
//! │                                          │
//! │  ┌─ listview ─────────────────────────┐  │
//! │  │ Process Name          PID          │  │
//! │  ├────────────────────────────────────┤  │
//! │  │ explorer.exe          404          │  │
//! │  │ taskmgr.exe           1145140      │  │
//! │  │ ...                                │  │
//! │  └────────────────────────────────────┘  │
//! └──────────────────────────────────────────┘
//! ```
//!
//! # Module Structure
//! - [`build`]   — constructs all controls (tab page, edit, listview)
//! - [`event`]   — registers all event handlers (resize, cue banner, process auto-refresh, context menu)
//! - [`layout`]  — DPI-aware layout constants, dynamic font metrics height calculation, and position calculators
//! - [`menu`]    — popup context menu construction and command definitions for ListView items
//! - [`page`]    — [`WindowVisualStylesPage`] struct: public API for construction
//! - [`process`] — system process snapshot queries and sorting logic powered by `sysinfo`

mod build;
mod event;
mod layout;
pub mod menu;
mod page;
pub mod process;

pub use page::WindowVisualStylesPage;
