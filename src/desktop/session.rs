//! Active interactive desktop session and security context resolution.
//!
//! When running under TrustedInstaller or SYSTEM credentials, standard user-specific
//! environment paths and registry branches resolve to `systemprofile`. This module queries
//! the active interactive desktop session's user token to obtain the true session user's
//! Security Identifier (SID) and locate their corresponding hive in `HKEY_USERS`.

use anyhow::{Context, Result, bail};
use rust_i18n::t;
use windows::{
    Win32::{
        Foundation::{CloseHandle, HANDLE, HLOCAL, LocalFree},
        Security::{
            Authorization::ConvertSidToStringSidW, GetTokenInformation, PSID, TOKEN_USER, TokenUser,
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

/// RAII guard releasing a [`PWSTR`] buffer allocated by Win32 functions via [`LocalFree`].
struct LocalAllocatedStringGuard {
    allocated_pwstr: PWSTR,
}

impl LocalAllocatedStringGuard {
    fn new(allocated_pwstr: PWSTR) -> Self {
        Self { allocated_pwstr }
    }

    /// Convert the inner null-terminated wide string into a standard Rust [`String`].
    ///
    /// Consumes the guard so the raw pointer cannot be accessed after conversion.
    /// The buffer is freed via [`Drop`] when this function returns.
    fn into_string(self) -> Result<String> {
        if self.allocated_pwstr.is_null() {
            bail!("{}", t!("ERROR_NULL_STRING_POINTER"));
        }

        unsafe {
            self.allocated_pwstr
                .to_string()
                .context(t!("ERROR_INVALID_UTF16_SID"))
        }
    }
}

impl Drop for LocalAllocatedStringGuard {
    fn drop(&mut self) {
        if !self.allocated_pwstr.is_null() {
            unsafe {
                let _ = LocalFree(HLOCAL(self.allocated_pwstr.0.cast()));
            }
        }
    }
}

/// Convert a raw binary [`PSID`] into its standard string representation.
fn convert_sid_to_string(binary_sid: PSID) -> Result<String> {
    let mut sid_pwstr = PWSTR::null();
    unsafe {
        ConvertSidToStringSidW(binary_sid, &mut sid_pwstr)
            .context(t!("ERROR_CONVERT_SID_TO_STRING_FAILED"))?;
    }

    LocalAllocatedStringGuard::new(sid_pwstr).into_string()
}

/// Resolve the active desktop session's user SID string (e.g. `"S-1-5-21-..."`).
pub fn resolve_active_user_sid() -> Result<String> {
    let mut session_id = 0;
    unsafe {
        ProcessIdToSessionId(GetCurrentProcessId(), &mut session_id)
            .context(t!("ERROR_GET_PROCESS_SESSION_ID_FAILED"))?;
    }

    let mut user_token = HANDLE::default();
    unsafe {
        WTSQueryUserToken(session_id, &mut user_token)
            .context(t!("ERROR_QUERY_USER_TOKEN_FAILED"))?;
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
        .context(t!("ERROR_GET_TOKEN_USER_FAILED"))?;
    }

    let token_user = unsafe { &*(token_buffer.as_ptr().cast::<TOKEN_USER>()) };
    convert_sid_to_string(token_user.User.Sid)
}

/// Open the root `HKEY_USERS\<Active-User-SID>` registry key for the current interactive user.
pub fn open_active_user_registry_root() -> Result<RegKey> {
    let user_sid = resolve_active_user_sid()?;
    let root_users_key = RegKey::predef(HKEY_USERS);
    root_users_key
        .open_subkey(&user_sid)
        .with_context(|| t!("ERROR_OPEN_USER_REGISTRY_ROOT_FAILED", user_sid = user_sid))
}
