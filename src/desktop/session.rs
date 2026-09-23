//! Active interactive desktop session and security context resolution.
//!
//! When running under TrustedInstaller or SYSTEM credentials, standard user-specific
//! environment paths and registry branches resolve to `systemprofile`. This module queries
//! the active interactive desktop session's user token to obtain the true session user's
//! Security Identifier (SID) and locate their corresponding hive in `HKEY_USERS`.

use std::ffi::c_void;

use anyhow::{Context, Result, bail};
use rust_i18n::t;
use windows::{
    Win32::{
        Foundation::{CloseHandle, HANDLE, HLOCAL, LocalFree},
        Security::{
            Authorization::ConvertSidToStringSidW, GetTokenInformation, TOKEN_USER, TokenUser,
        },
        System::{
            RemoteDesktop::{ProcessIdToSessionId, WTSQueryUserToken},
            Threading::GetCurrentProcessId,
        },
    },
    core::PWSTR,
};
use winreg::{RegKey, enums::HKEY_USERS};

/// RAII guard for an OS [`HANDLE`] ensuring automatic closure on drop.
struct HandleGuard {
    raw_handle: HANDLE,
}

impl HandleGuard {
    fn new(raw_handle: HANDLE) -> Self {
        Self { raw_handle }
    }

    fn as_raw(&self) -> HANDLE {
        self.raw_handle
    }
}

impl Drop for HandleGuard {
    fn drop(&mut self) {
        if !self.raw_handle.is_invalid() {
            unsafe {
                let _ = CloseHandle(self.raw_handle);
            }
        }
    }
}

/// RAII guard for local security memory allocated by Win32 security APIs.
struct LocalMemoryGuard {
    pointer: *mut c_void,
}

impl Drop for LocalMemoryGuard {
    fn drop(&mut self) {
        if !self.pointer.is_null() {
            unsafe {
                let _ = LocalFree(HLOCAL(self.pointer));
            }
        }
    }
}

/// Resolve the active desktop session's user SID string (e.g. `"S-1-5-21-..."`).
pub fn resolve_active_user_sid() -> Result<String> {
    let mut session_id = 0;
    unsafe {
        ProcessIdToSessionId(GetCurrentProcessId(), &mut session_id)
            .context("Failed to obtain current process session ID")?;
    }

    let mut user_token = HANDLE::default();
    unsafe {
        WTSQueryUserToken(session_id, &mut user_token)
            .context("Failed to query user token for active desktop session")?;
    }
    let token_guard = HandleGuard::new(user_token);

    let mut return_length = 0;
    let _ = unsafe {
        GetTokenInformation(token_guard.as_raw(), TokenUser, None, 0, &mut return_length)
    };
    if return_length == 0 {
        bail!("{}", t!("ERROR_FAILED_TO_RESOLVE_USER_SID"));
    }

    let mut token_buffer = vec![0u8; return_length as usize];
    unsafe {
        GetTokenInformation(
            token_guard.as_raw(),
            TokenUser,
            Some(token_buffer.as_mut_ptr().cast()),
            return_length,
            &mut return_length,
        )
        .context("Failed to get token user information")?;
    }

    let token_user = unsafe { &*(token_buffer.as_ptr().cast::<TOKEN_USER>()) };
    let mut sid_pwstr = PWSTR::null();
    unsafe {
        ConvertSidToStringSidW(token_user.User.Sid, &mut sid_pwstr)
            .context("Failed to convert binary SID to string representation")?;
    }

    let _local_guard = LocalMemoryGuard {
        pointer: sid_pwstr.0.cast(),
    };

    let user_sid = unsafe {
        sid_pwstr
            .to_string()
            .context("Invalid UTF-16 sequence in SID string")?
    };

    Ok(user_sid)
}

/// Open the root `HKEY_USERS\<Active-User-SID>` registry key for the current interactive user.
pub fn open_active_user_registry_root() -> Result<RegKey> {
    let user_sid = resolve_active_user_sid()?;
    let root_users_key = RegKey::predef(HKEY_USERS);
    root_users_key
        .open_subkey(&user_sid)
        .with_context(|| format!("Failed to open registry root for user SID: {user_sid}"))
}
