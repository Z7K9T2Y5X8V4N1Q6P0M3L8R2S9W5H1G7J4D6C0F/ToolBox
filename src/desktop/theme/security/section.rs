//! NT Named Section kernel object abstraction.
//!
//! Provides a safe interface for opening and querying/modifying security attributes
//! of Windows kernel section objects via native NT system calls.

use std::{alloc::Layout, mem};

use anyhow::{Result, anyhow, bail};
use windows::{
    Wdk::{Foundation::OBJECT_ATTRIBUTES, System::Memory::NtOpenSection},
    Win32::{
        Foundation::{CloseHandle, HANDLE, STATUS_SUCCESS, UNICODE_STRING},
        Security::{
            Authorization::{
                ConvertSecurityDescriptorToStringSecurityDescriptorW, SDDL_REVISION_1,
            },
            DACL_SECURITY_INFORMATION, GetKernelObjectSecurity, PSECURITY_DESCRIPTOR,
            SetKernelObjectSecurity,
        },
        Storage::FileSystem::{READ_CONTROL, WRITE_DAC},
        System::Kernel::OBJ_CASE_INSENSITIVE,
    },
    core::PWSTR,
};

use super::sddl::{SddlStringGuard, SecurityDescriptorGuard};

/// Dynamically allocated buffer guaranteed to satisfy strict pointer alignment for Win32 security descriptors.
struct AlignedSecurityBuffer {
    base_address: *mut u8,
    memory_layout: Layout,
}

impl Drop for AlignedSecurityBuffer {
    fn drop(&mut self) {
        if !self.base_address.is_null() && self.memory_layout.size() > 0 {
            unsafe {
                std::alloc::dealloc(self.base_address, self.memory_layout);
            }
        }
    }
}

impl AlignedSecurityBuffer {
    /// Allocate an aligned zero-initialized buffer satisfying pointer alignment.
    fn allocate(buffer_capacity: usize) -> Option<Self> {
        let memory_layout =
            Layout::from_size_align(buffer_capacity, mem::align_of::<usize>()).ok()?;
        let base_address = unsafe { std::alloc::alloc_zeroed(memory_layout) };
        if base_address.is_null() {
            None
        } else {
            Some(Self {
                base_address,
                memory_layout,
            })
        }
    }

    fn as_security_descriptor(&self) -> PSECURITY_DESCRIPTOR {
        PSECURITY_DESCRIPTOR(self.base_address.cast())
    }
}

/// RAII handle managing an open native kernel section object.
pub struct KernelSectionHandle {
    raw_handle: HANDLE,
}

impl Drop for KernelSectionHandle {
    fn drop(&mut self) {
        if !self.raw_handle.is_invalid() {
            unsafe {
                let _ = CloseHandle(self.raw_handle);
            }
        }
    }
}

impl KernelSectionHandle {
    /// Open a session kernel section object with DACL inspection rights.
    pub fn open_for_query(session_id: u32, section_name: &str) -> Result<Self> {
        let object_nt_path = format!("\\Sessions\\{session_id}\\Windows\\{section_name}");
        Self::open_nt_section(&object_nt_path, READ_CONTROL.0)
    }

    /// Open a session kernel section object with DACL modification rights.
    pub fn open_for_modification(session_id: u32, section_name: &str) -> Result<Self> {
        let object_nt_path = format!("\\Sessions\\{session_id}\\Windows\\{section_name}");
        Self::open_nt_section(&object_nt_path, WRITE_DAC.0)
    }

    /// Query the current DACL security descriptor of this section object and convert directly to an SDDL string guard.
    pub fn query_dacl_sddl(&self) -> Result<SddlStringGuard> {
        let mut required_security_descriptor_size = 0u32;
        let _ = unsafe {
            GetKernelObjectSecurity(
                self.raw_handle,
                DACL_SECURITY_INFORMATION.0 as u32,
                PSECURITY_DESCRIPTOR(std::ptr::null_mut()),
                0,
                &mut required_security_descriptor_size,
            )
        };

        if required_security_descriptor_size == 0 {
            bail!("Failed to query required security descriptor buffer length for kernel section");
        }

        // Use pointer-aligned heap buffer instead of unaligned Vec<u8> to prevent alignment UB.
        let security_descriptor_payload =
            AlignedSecurityBuffer::allocate(required_security_descriptor_size as usize)
                .ok_or_else(|| {
                    anyhow!("Failed to allocate aligned buffer for security descriptor")
                })?;

        let query_security_result = unsafe {
            GetKernelObjectSecurity(
                self.raw_handle,
                DACL_SECURITY_INFORMATION.0 as u32,
                security_descriptor_payload.as_security_descriptor(),
                required_security_descriptor_size,
                &mut required_security_descriptor_size,
            )
        };

        if let Err(query_security_error) = query_security_result {
            bail!("GetKernelObjectSecurity failed on kernel section: {query_security_error}");
        }

        let mut allocated_sddl_pointer = PWSTR::null();
        let convert_result = unsafe {
            ConvertSecurityDescriptorToStringSecurityDescriptorW(
                security_descriptor_payload.as_security_descriptor(),
                SDDL_REVISION_1,
                DACL_SECURITY_INFORMATION,
                &mut allocated_sddl_pointer,
                None,
            )
        };

        if let Err(convert_error) = convert_result {
            bail!("ConvertSecurityDescriptorToStringSecurityDescriptorW failed: {convert_error}");
        }

        Ok(SddlStringGuard::from_raw(allocated_sddl_pointer))
    }

    /// Apply an updated DACL security descriptor to this section object.
    pub fn set_dacl_security_descriptor(
        &self,
        security_descriptor: &SecurityDescriptorGuard,
    ) -> Result<()> {
        let apply_security_result = unsafe {
            SetKernelObjectSecurity(
                self.raw_handle,
                DACL_SECURITY_INFORMATION,
                security_descriptor.as_raw(),
            )
        };

        if let Err(apply_security_error) = apply_security_result {
            bail!("SetKernelObjectSecurity failed on kernel section: {apply_security_error}");
        }

        Ok(())
    }

    /// Open an arbitrary NT kernel section path using `NtOpenSection`.
    fn open_nt_section(section_nt_path: &str, desired_access_mask: u32) -> Result<Self> {
        let mut encoded_wide_path: Vec<u16> = section_nt_path.encode_utf16().collect();
        let path_length_in_bytes = u16::try_from(encoded_wide_path.len().saturating_mul(2))
            .map_err(|_| anyhow!("Kernel section path length overflows u16 range"))?;

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

        let mut section_handle = HANDLE::default();
        let nt_status =
            unsafe { NtOpenSection(&mut section_handle, desired_access_mask, &object_attributes) };

        if nt_status != STATUS_SUCCESS {
            bail!(
                "NtOpenSection on \"{section_nt_path}\" failed with NTSTATUS: {:#010X}",
                nt_status.0
            );
        }

        Ok(Self {
            raw_handle: section_handle,
        })
    }
}
