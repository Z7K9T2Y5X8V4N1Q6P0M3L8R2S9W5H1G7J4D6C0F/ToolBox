//! Active interactive desktop session and security context resolution.
//!
//! When running under TrustedInstaller or SYSTEM credentials, standard user-specific
//! environment paths and registry branches resolve to `systemprofile`. This module queries
//! the active interactive desktop session's user token to obtain the true session user's
//! Security Identifier (SID) and locate their corresponding hive in `HKEY_USERS`.

use anyhow::{Context, Result};
use elevate_ti::ProcessToken;
use rust_i18n::t;
use windows::Win32::{
    Foundation::HANDLE,
    System::{
        RemoteDesktop::{ProcessIdToSessionId, WTSQueryUserToken},
        Threading::GetCurrentProcessId,
    },
};
use winreg::{RegKey, enums::HKEY_USERS};

/// Query the terminal services session identifier for the current process.
fn fetch_current_session_id() -> Result<u32> {
    let mut session_id = 0;
    unsafe {
        ProcessIdToSessionId(GetCurrentProcessId(), &mut session_id)
            .context(t!("ERROR_GET_PROCESS_SESSION_ID_FAILED"))?;
    }
    Ok(session_id)
}

/// Query the primary interactive user token for a given session ID.
fn fetch_session_user_token(session_id: u32) -> Result<ProcessToken> {
    let mut user_token = HANDLE::default();
    unsafe {
        WTSQueryUserToken(session_id, &mut user_token)
            .context(t!("ERROR_QUERY_USER_TOKEN_FAILED"))?;
    }
    Ok(ProcessToken::from_raw_handle(user_token))
}

/// Resolve the active desktop session's user SID string (e.g. `"S-1-5-21-..."`).
pub fn resolve_active_user_sid() -> Result<String> {
    let session_id = fetch_current_session_id()?;
    let user_token = fetch_session_user_token(session_id)?;
    user_token
        .query_user_sid_string()
        .context(t!("ERROR_FAILED_TO_RESOLVE_USER_SID"))
}

/// Open the root `HKEY_USERS\<Active-User-SID>` registry key for the current interactive user.
pub fn open_active_user_registry_root() -> Result<RegKey> {
    let user_sid = resolve_active_user_sid()?;
    let root_users_key = RegKey::predef(HKEY_USERS);
    root_users_key
        .open_subkey(&user_sid)
        .with_context(|| t!("ERROR_OPEN_USER_REGISTRY_ROOT_FAILED", user_sid = user_sid))
}
