//! Security descriptor and SDDL memory RAII wrappers.
//!
//! Provides memory-safe encapsulation for native Windows security descriptors
//! and wide SDDL string buffers returned by Windows authorization APIs.

use anyhow::{Context, Result, bail};
use windows::{
    Win32::{
        Foundation::{HLOCAL, LocalFree},
        Security::{
            Authorization::{
                ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
            },
            PSECURITY_DESCRIPTOR,
        },
    },
    core::{PCWSTR, PWSTR},
};

/// The access permission policy inferred from the session's ThemeSection DACL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeSectionAccessPolicy {
    /// Discretionary access contains explicit deny entries for Everyone (`WD`).
    ExplicitlyDenied,
    /// Discretionary access explicitly grants access to Everyone (`WD`) without denials.
    ExplicitlyAllowed,
    /// Access policy could not be conclusively resolved from standard templates.
    Unrecognized,
}

/// RAII guard managing native heap memory allocated for a [`PSECURITY_DESCRIPTOR`].
pub struct SecurityDescriptorGuard {
    descriptor_pointer: PSECURITY_DESCRIPTOR,
}

impl Drop for SecurityDescriptorGuard {
    fn drop(&mut self) {
        if !self.descriptor_pointer.is_invalid() {
            unsafe {
                let _ = LocalFree(HLOCAL(self.descriptor_pointer.0));
            }
        }
    }
}

impl SecurityDescriptorGuard {
    /// Encapsulate a raw security descriptor pointer inside an RAII lifecycle manager.
    pub const fn from_raw(descriptor_pointer: PSECURITY_DESCRIPTOR) -> Self {
        Self { descriptor_pointer }
    }

    /// Parse an SDDL string representation into an allocated security descriptor.
    pub fn from_sddl(target_sddl_definition: PCWSTR) -> Result<Self> {
        let mut descriptor_pointer = PSECURITY_DESCRIPTOR::default();
        let conversion_result = unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                target_sddl_definition,
                SDDL_REVISION_1,
                &mut descriptor_pointer,
                None,
            )
        };

        if let Err(conversion_error) = conversion_result {
            bail!(
                "ConvertStringSecurityDescriptorToSecurityDescriptorW failed: {conversion_error}"
            );
        }

        Ok(Self::from_raw(descriptor_pointer))
    }

    /// Access the underlying raw security descriptor pointer for Win32 API interop.
    pub const fn as_raw(&self) -> PSECURITY_DESCRIPTOR {
        self.descriptor_pointer
    }
}

/// RAII guard releasing a [`PWSTR`] buffer allocated by Win32 authorization routines.
pub struct SddlStringGuard {
    allocated_sddl_pointer: PWSTR,
}

impl Drop for SddlStringGuard {
    fn drop(&mut self) {
        if !self.allocated_sddl_pointer.is_null() {
            unsafe {
                let _ = LocalFree(HLOCAL(self.allocated_sddl_pointer.0.cast()));
            }
        }
    }
}

impl SddlStringGuard {
    /// Encapsulate a raw wide string pointer into an auto-disposing guard.
    pub const fn from_raw(allocated_sddl_pointer: PWSTR) -> Self {
        Self {
            allocated_sddl_pointer,
        }
    }

    /// Convert the native buffer into an owned Rust [`String`], immediately dropping the guard.
    pub fn into_string(self) -> Result<String> {
        if self.allocated_sddl_pointer.is_null() {
            return Ok(String::new());
        }
        unsafe { self.allocated_sddl_pointer.to_string() }
            .context("Failed to decode UTF-16 SDDL buffer into UTF-8")
    }
}

/// Parse an SDDL string representation and evaluate the ThemeSection access control status.
///
/// Inspects individual ACE tokens between parentheses rather than performing ad-hoc
/// substring queries, ensuring that access denial explicitly targets the expected principals.
pub fn evaluate_theme_section_policy(sddl_definition: &str) -> ThemeSectionAccessPolicy {
    let mut has_deny_entry_for_everyone = false;
    let mut has_allow_entry_for_everyone = false;

    // SDDL ACEs are formatted within parentheses, e.g. "(D;OICI;GA;;;WD)"
    for ace_entry in sddl_definition.split('(') {
        let Some(ace_payload) = ace_entry.split(')').next() else {
            continue;
        };

        let ace_fields: Vec<&str> = ace_payload.split(';').collect();
        if ace_fields.len() != 6 {
            continue;
        }

        let ace_type = match ace_fields.first() {
            Some(ace_type) => *ace_type,
            None => continue,
        };

        let trustee_sid = match ace_fields.get(5) {
            Some(trustee_sid) => *trustee_sid,
            None => continue,
        };

        if trustee_sid == "WD" {
            match ace_type {
                "D" => has_deny_entry_for_everyone = true,
                "A" => has_allow_entry_for_everyone = true,
                _ => {}
            }
        }
    }

    if has_deny_entry_for_everyone {
        ThemeSectionAccessPolicy::ExplicitlyDenied
    } else if has_allow_entry_for_everyone {
        ThemeSectionAccessPolicy::ExplicitlyAllowed
    } else {
        ThemeSectionAccessPolicy::Unrecognized
    }
}
