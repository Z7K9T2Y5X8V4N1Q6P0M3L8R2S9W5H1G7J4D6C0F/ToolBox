//! Low-level kernel and security primitives for theme management.
//!
//! # Module Structure
//! - [`hook`]    — RAII management for Windows accessibility event hooks
//! - [`sddl`]    — security descriptor and SDDL string memory management
//! - [`section`] — NT named kernel section object manipulation

pub mod hook;
pub mod sddl;
pub mod section;

pub use hook::WinEventHookGuard;
pub use sddl::{SecurityDescriptorGuard, ThemeSectionAccessPolicy, evaluate_theme_section_policy};
pub use section::KernelSectionHandle;
