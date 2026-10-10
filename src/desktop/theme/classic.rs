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

use anyhow::{Context, Result, anyhow, bail};
use windows::{
    Wdk::{Foundation::OBJECT_ATTRIBUTES, System::Memory::NtOpenSection},
    Win32::{
        Foundation::{
            CloseHandle, HANDLE, HLOCAL, LPARAM, LocalFree, STATUS_SUCCESS, UNICODE_STRING, WPARAM,
        },
        Security::{
            Authorization::{
                ConvertSecurityDescriptorToStringSecurityDescriptorW,
                ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
            },
            DACL_SECURITY_INFORMATION, GetKernelObjectSecurity, PSECURITY_DESCRIPTOR,
            SetKernelObjectSecurity,
        },
        Storage::FileSystem::{READ_CONTROL, WRITE_DAC},
        System::Kernel::OBJ_CASE_INSENSITIVE,
        UI::WindowsAndMessaging::{HWND_BROADCAST, PostMessageW, WM_THEMECHANGED},
    },
    core::{PCWSTR, PWSTR, w},
};

use crate::desktop::session;

/// SDDL string denying read permissions to starve the visual styles subsystem.
const SDDL_DENY_THEME_SECTION: PCWSTR =
    w!("D:(D;OICI;GA;;;WD)(D;OICI;GA;;;AN)(D;OICI;GA;;;AU)(D;OICI;GA;;;BA)");

/// SDDL string granting permissions to restore standard visual style rendering.
const SDDL_ALLOW_THEME_SECTION: PCWSTR =
    w!("D:(A;OICI;GA;;;WD)(A;OICI;GA;;;AN)(A;OICI;GA;;;AU)(A;OICI;GA;;;BA)");

/// RAII wrapper for native [`HANDLE`] to guarantee disposal.
struct KernelObjectHandleGuard {
    object_handle: HANDLE,
}

impl KernelObjectHandleGuard {
    const fn new(object_handle: HANDLE) -> Self {
        Self { object_handle }
    }

    const fn raw(&self) -> HANDLE {
        self.object_handle
    }
}

impl Drop for KernelObjectHandleGuard {
    fn drop(&mut self) {
        if !self.object_handle.is_invalid() {
            unsafe {
                let _ = CloseHandle(self.object_handle);
            }
        }
    }
}

/// RAII wrapper for heap memory returned by [`ConvertStringSecurityDescriptorToSecurityDescriptorW`].
struct SecurityDescriptorMemoryGuard {
    security_descriptor_pointer: PSECURITY_DESCRIPTOR,
}

impl SecurityDescriptorMemoryGuard {
    const fn new(security_descriptor_pointer: PSECURITY_DESCRIPTOR) -> Self {
        Self {
            security_descriptor_pointer,
        }
    }

    const fn raw(&self) -> PSECURITY_DESCRIPTOR {
        self.security_descriptor_pointer
    }
}

impl Drop for SecurityDescriptorMemoryGuard {
    fn drop(&mut self) {
        if !self.security_descriptor_pointer.is_invalid() {
            unsafe {
                let _ = LocalFree(HLOCAL(self.security_descriptor_pointer.0));
            }
        }
    }
}

/// RAII wrapper for wide string memory allocated by [`ConvertSecurityDescriptorToStringSecurityDescriptorW`].
struct SddlStringMemoryGuard {
    raw_sddl_string_pointer: PWSTR,
}

impl SddlStringMemoryGuard {
    const fn new(raw_sddl_string_pointer: PWSTR) -> Self {
        Self {
            raw_sddl_string_pointer,
        }
    }

    fn to_string_lossy(&self) -> String {
        if self.raw_sddl_string_pointer.is_null() {
            String::new()
        } else {
            unsafe { self.raw_sddl_string_pointer.to_string().unwrap_or_default() }
        }
    }
}

impl Drop for SddlStringMemoryGuard {
    fn drop(&mut self) {
        if !self.raw_sddl_string_pointer.is_null() {
            unsafe {
                let _ = LocalFree(HLOCAL(self.raw_sddl_string_pointer.0 as *mut _));
            }
        }
    }
}

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
        match read_theme_section_sddl() {
            Ok(sddl_text) => {
                if sddl_text.contains("(D;") {
                    GlobalClassicStylesState::Enabled
                } else {
                    GlobalClassicStylesState::Disabled
                }
            }
            Err(read_sddl_error) => {
                let read_sddl_error_message = read_sddl_error.to_string();
                log::warn!(
                    "Failed to read ThemeSection DACL, defaulting to Disabled: {error_message}",
                    error_message = read_sddl_error_message
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

        apply_theme_section_security_sddl(SDDL_DENY_THEME_SECTION)
            .context("Failed to apply deny DACL to ThemeSection")?;
        notify_theme_changed();
        Ok(())
    }

    /// Disable classic styles by restoring permissions to `ThemeSection`.
    pub fn disable(&self) -> Result<()> {
        if self.current_state() == GlobalClassicStylesState::Disabled {
            return Ok(());
        }

        apply_theme_section_security_sddl(SDDL_ALLOW_THEME_SECTION)
            .context("Failed to apply allow DACL to ThemeSection")?;
        notify_theme_changed();
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

// ---------------------------------------------------------------------------
// Security Descriptor & Kernel Section Manipulation
// ---------------------------------------------------------------------------

/// Post WM_THEMECHANGED to trigger applications to re-evaluate ThemeSection permissions.
fn notify_theme_changed() {
    unsafe {
        let _ = PostMessageW(HWND_BROADCAST, WM_THEMECHANGED, WPARAM(0), LPARAM(0));
    }
}

/// Read the current DACL of the session's `ThemeSection` object and convert it to an SDDL string.
fn read_theme_section_sddl() -> Result<String> {
    let current_session_id = session::fetch_current_session_id()
        .context("Failed to query current desktop session ID")?;

    let theme_section_path = format!("\\Sessions\\{current_session_id}\\Windows\\ThemeSection");
    let theme_section_handle_guard =
        open_kernel_theme_section(&theme_section_path, READ_CONTROL.0)?;

    let mut required_buffer_length = 0;
    let _ = unsafe {
        GetKernelObjectSecurity(
            theme_section_handle_guard.raw(),
            DACL_SECURITY_INFORMATION.0 as u32,
            PSECURITY_DESCRIPTOR(std::ptr::null_mut()),
            0,
            &mut required_buffer_length,
        )
    };

    if required_buffer_length == 0 {
        bail!("Failed to query required security descriptor buffer length for ThemeSection");
    }

    let mut security_descriptor_buffer = vec![0u8; required_buffer_length as usize];
    let query_security_result = unsafe {
        GetKernelObjectSecurity(
            theme_section_handle_guard.raw(),
            DACL_SECURITY_INFORMATION.0 as u32,
            PSECURITY_DESCRIPTOR(security_descriptor_buffer.as_mut_ptr().cast()),
            required_buffer_length,
            &mut required_buffer_length,
        )
    };

    if let Err(query_security_error) = query_security_result {
        bail!("GetKernelObjectSecurity failed on ThemeSection: {query_security_error}");
    }

    let mut raw_sddl_string_pointer = PWSTR::null();
    let convert_result = unsafe {
        ConvertSecurityDescriptorToStringSecurityDescriptorW(
            PSECURITY_DESCRIPTOR(security_descriptor_buffer.as_mut_ptr().cast()),
            SDDL_REVISION_1,
            DACL_SECURITY_INFORMATION,
            &mut raw_sddl_string_pointer,
            None,
        )
    };

    if let Err(convert_error) = convert_result {
        bail!("ConvertSecurityDescriptorToStringSecurityDescriptorW failed: {convert_error}");
    }

    let sddl_memory_guard = SddlStringMemoryGuard::new(raw_sddl_string_pointer);
    Ok(sddl_memory_guard.to_string_lossy())
}

/// Alter the DACL of the current session's `ThemeSection` object.
fn apply_theme_section_security_sddl(target_sddl_string: PCWSTR) -> Result<()> {
    let current_session_id = session::fetch_current_session_id()
        .context("Failed to query current desktop session ID")?;

    let theme_section_path = format!("\\Sessions\\{current_session_id}\\Windows\\ThemeSection");
    let theme_section_handle_guard = open_kernel_theme_section(&theme_section_path, WRITE_DAC.0)?;

    let security_descriptor_guard = parse_sddl_into_security_descriptor(target_sddl_string)?;

    let apply_security_result = unsafe {
        SetKernelObjectSecurity(
            theme_section_handle_guard.raw(),
            DACL_SECURITY_INFORMATION,
            security_descriptor_guard.raw(),
        )
    };

    if let Err(system_error) = apply_security_result {
        bail!("SetKernelObjectSecurity failed on ThemeSection: {system_error}");
    }

    Ok(())
}

/// Convert an SDDL string into an allocated security descriptor wrapped in an RAII guard.
fn parse_sddl_into_security_descriptor(
    sddl_string: PCWSTR,
) -> Result<SecurityDescriptorMemoryGuard> {
    let mut security_descriptor_pointer = PSECURITY_DESCRIPTOR::default();

    let conversion_result = unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl_string,
            SDDL_REVISION_1,
            &mut security_descriptor_pointer,
            None,
        )
    };

    if let Err(conversion_error) = conversion_result {
        bail!("ConvertStringSecurityDescriptorToSecurityDescriptorW failed: {conversion_error}");
    }

    Ok(SecurityDescriptorMemoryGuard::new(
        security_descriptor_pointer,
    ))
}

/// Open the `ThemeSection` kernel named section with the requested access mask via [`NtOpenSection`].
fn open_kernel_theme_section(
    theme_section_nt_path: &str,
    desired_access_mask: u32,
) -> Result<KernelObjectHandleGuard> {
    let mut encoded_wide_path: Vec<u16> = theme_section_nt_path.encode_utf16().collect();
    let path_length_in_bytes = u16::try_from(encoded_wide_path.len().saturating_mul(2))
        .map_err(|_| anyhow!("Theme section path length overflows u16 range"))?;

    let unicode_section_name = UNICODE_STRING {
        Length: path_length_in_bytes,
        MaximumLength: path_length_in_bytes,
        Buffer: PWSTR(encoded_wide_path.as_mut_ptr()),
    };

    let object_attributes = OBJECT_ATTRIBUTES {
        Length: size_of::<OBJECT_ATTRIBUTES>() as u32,
        RootDirectory: HANDLE::default(),
        ObjectName: std::ptr::from_ref(&unicode_section_name),
        Attributes: OBJ_CASE_INSENSITIVE as u32,
        SecurityDescriptor: std::ptr::null_mut(),
        SecurityQualityOfService: std::ptr::null_mut(),
    };

    let mut section_raw_handle = HANDLE::default();
    let nt_status = unsafe {
        NtOpenSection(
            &mut section_raw_handle,
            desired_access_mask,
            &object_attributes,
        )
    };

    if nt_status != STATUS_SUCCESS {
        bail!(
            "NtOpenSection on \"{theme_section_nt_path}\" failed with NTSTATUS: {:#010X}",
            nt_status.0
        );
    }

    Ok(KernelObjectHandleGuard::new(section_raw_handle))
}
