//! Global Windows Classic visual style switching engine.
//!
//! Starves the Windows theme engine (`uxtheme.dll`) by altering the Discretionary
//! Access Control List (DACL) on the kernel session object
//! `\Sessions\<SessionID>\Windows\ThemeSection`.
//!
//! When access is denied, components attempting to query visual style resources
//! encounter `STATUS_ACCESS_DENIED` and fall back to the built-in Windows Classic
//! 3D control rendering pipeline.
//!
//! The active state is directly inferred from the kernel object's actual DACL
//! rather than relying on an in-memory flag.

use anyhow::{Context, Result};
use windows::{
    Win32::{
        Foundation::{LPARAM, WPARAM},
        UI::WindowsAndMessaging::{HWND_BROADCAST, PostMessageW, WM_THEMECHANGED},
    },
    core::{PCWSTR, w},
};

use crate::desktop::{
    session,
    theme::security::{
        KernelSectionHandle, SecurityDescriptorGuard, ThemeSectionAccessPolicy,
        evaluate_theme_section_policy,
    },
};

/// SDDL string denying read permissions to starve the visual styles subsystem.
const SDDL_DENY_THEME_SECTION: PCWSTR =
    w!("D:(D;OICI;GA;;;WD)(D;OICI;GA;;;AN)(D;OICI;GA;;;AU)(D;OICI;GA;;;BA)");

/// SDDL string granting permissions to restore standard visual style rendering.
const SDDL_ALLOW_THEME_SECTION: PCWSTR =
    w!("D:(A;OICI;GA;;;WD)(A;OICI;GA;;;AN)(A;OICI;GA;;;AU)(A;OICI;GA;;;BA)");

/// Target name of the kernel theme section object under the session directory.
const THEME_SECTION_OBJECT_NAME: &str = "ThemeSection";

/// Operational state of the global classic theme mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalClassicStylesState {
    /// Classic styles active (ThemeSection access is denied).
    Enabled,
    /// Classic styles inactive (ThemeSection access is restored).
    Disabled,
}

/// Lifecycle controller for managing global Classic appearance.
#[derive(Default)]
pub struct GlobalClassicStylesWatcher;

impl GlobalClassicStylesWatcher {
    /// Create an unstarted classic styles controller.
    pub const fn new() -> Self {
        Self
    }

    /// Retrieve the current operational state directly from the kernel `ThemeSection` object's DACL.
    pub fn current_state(&self) -> GlobalClassicStylesState {
        match fetch_theme_section_policy() {
            Ok(ThemeSectionAccessPolicy::ExplicitlyDenied) => GlobalClassicStylesState::Enabled,
            Ok(ThemeSectionAccessPolicy::ExplicitlyAllowed) => GlobalClassicStylesState::Disabled,
            Ok(ThemeSectionAccessPolicy::Unrecognized) => {
                log::warn!("ThemeSection DACL is unrecognized, defaulting to Disabled");
                GlobalClassicStylesState::Disabled
            }
            Err(fetch_policy_error) => {
                let fetch_policy_error_message = fetch_policy_error.to_string();
                log::warn!(
                    "Failed to read ThemeSection DACL, defaulting to Disabled: {error_message}",
                    error_message = fetch_policy_error_message
                );
                GlobalClassicStylesState::Disabled
            }
        }
    }

    /// Enable classic styles by denying access to `ThemeSection`.
    pub fn enable(&self) -> Result<()> {
        if self.current_state() == GlobalClassicStylesState::Enabled {
            return Ok(());
        }

        apply_theme_section_policy(SDDL_DENY_THEME_SECTION)
            .context("Failed to apply deny DACL to ThemeSection")?;
        notify_system_theme_changed();
        Ok(())
    }

    /// Disable classic styles by restoring permissions to `ThemeSection`.
    pub fn disable(&self) -> Result<()> {
        if self.current_state() == GlobalClassicStylesState::Disabled {
            return Ok(());
        }

        apply_theme_section_policy(SDDL_ALLOW_THEME_SECTION)
            .context("Failed to apply allow DACL to ThemeSection")?;
        notify_system_theme_changed();
        Ok(())
    }

    /// Toggle the running state between enabled and disabled based on the live kernel state.
    pub fn toggle(&self) -> Result<GlobalClassicStylesState> {
        match self.current_state() {
            GlobalClassicStylesState::Enabled => {
                self.disable()?;
                Ok(GlobalClassicStylesState::Disabled)
            }
            GlobalClassicStylesState::Disabled => {
                self.enable()?;
                Ok(GlobalClassicStylesState::Enabled)
            }
        }
    }
}

/// Inspect the session's `ThemeSection` object and resolve its security policy.
fn fetch_theme_section_policy() -> Result<ThemeSectionAccessPolicy> {
    let current_session_id = session::fetch_current_session_id()
        .context("Failed to query current desktop session ID")?;

    let section_handle =
        KernelSectionHandle::open_for_query(current_session_id, THEME_SECTION_OBJECT_NAME)?;
    let sddl_guard = section_handle.query_dacl_sddl()?;
    let sddl_payload = sddl_guard.into_string()?;

    Ok(evaluate_theme_section_policy(&sddl_payload))
}

/// Apply a target SDDL definition directly to the session's `ThemeSection` object.
fn apply_theme_section_policy(target_sddl_definition: PCWSTR) -> Result<()> {
    let current_session_id = session::fetch_current_session_id()
        .context("Failed to query current desktop session ID")?;

    let section_handle =
        KernelSectionHandle::open_for_modification(current_session_id, THEME_SECTION_OBJECT_NAME)?;
    let target_security_descriptor = SecurityDescriptorGuard::from_sddl(target_sddl_definition)?;

    section_handle.set_dacl_security_descriptor(&target_security_descriptor)
}

/// Broadcast `WM_THEMECHANGED` across all top-level windows to trigger theme re-evaluation.
fn notify_system_theme_changed() {
    unsafe {
        let _ = PostMessageW(HWND_BROADCAST, WM_THEMECHANGED, WPARAM(0), LPARAM(0));
    }
}
