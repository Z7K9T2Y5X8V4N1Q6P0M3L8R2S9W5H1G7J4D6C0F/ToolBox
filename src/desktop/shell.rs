//! Windows desktop shell process manipulation utilities.
//!
//! Provides routines to terminate and restart core Windows desktop shell processes (Explorer.exe).

use anyhow::{Context, Result, bail};
use elevate_ti::{ProcessSpawner, TokenType};
use rust_i18n::t;
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::Threading::{OpenProcess, PROCESS_TERMINATE, TerminateProcess},
    UI::WindowsAndMessaging::{GetShellWindow, GetWindowThreadProcessId},
};
use winreg::{
    RegKey,
    enums::{HKEY_LOCAL_MACHINE, KEY_QUERY_VALUE},
};

use crate::desktop::session;

/// Subkey path to the Winlogon configuration in HKLM and HKCU.
const WINLOGON_REGISTRY_PATH: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon";

/// Value name for the automatic shell restart setting.
const AUTO_RESTART_SHELL_VALUE_NAME: &str = "AutoRestartShell";

/// Value name for the configured desktop shell executable.
const SHELL_REGISTRY_VALUE_NAME: &str = "Shell";

/// Expected value indicating that Winlogon automatically restarts Explorer.exe.
const AUTO_RESTART_SHELL_ENABLED_VALUE: u32 = 1;

/// Default desktop shell command line fallback when registry keys are absent.
const DEFAULT_SHELL_COMMAND: &str = "explorer.exe";

/// Primary desktop window station path.
const INTERACTIVE_DESKTOP_PATH: &str = "WinSta0\\Default";

/// Restart strategy derived from inspecting the Winlogon configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShellRestartStrategy {
    /// Winlogon has `AutoRestartShell` enabled and will relaunch the shell automatically.
    AutomaticByWinlogon,
    /// `AutoRestartShell` is disabled or absent; the shell must be manually spawned as the interactive user.
    ManualSpawnAsUser,
}

/// RAII wrapper for a process handle to ensure proper closure via [`CloseHandle`].
struct ProcessHandleGuard {
    process_handle: HANDLE,
}

impl ProcessHandleGuard {
    const fn new(process_handle: HANDLE) -> Self {
        Self { process_handle }
    }

    const fn as_raw(&self) -> HANDLE {
        self.process_handle
    }
}

impl Drop for ProcessHandleGuard {
    fn drop(&mut self) {
        if !self.process_handle.is_invalid() {
            unsafe {
                let _ = CloseHandle(self.process_handle);
            }
        }
    }
}

/// Restart the Windows desktop shell (Explorer.exe) matching System Informer and Task Manager behavior.
///
/// # Operational Behavior
/// 1. Inspects whether `AutoRestartShell` is set to `1` in `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon`.
/// 2. If manual relaunch is required, queries the target shell executable from HKCU or HKLM,
///    falling back to `"explorer.exe"` if absent.
/// 3. Identifies the process owning the primary desktop shell window (`GetShellWindow`).
/// 4. Opens the target process with `PROCESS_TERMINATE` rights and terminates it.
/// 5. If `AutoRestartShell` was not `1`, manually spawns the shell under the interactive user token
///    via [`ProcessSpawner`].
///
/// # References
/// - [System Informer Repository](https://github.com/winsiderss/systeminformer)
///
/// # Errors
/// Returns an error if:
/// - The primary desktop shell window cannot be found.
/// - The shell process ID cannot be queried.
/// - The target process cannot be opened with termination rights.
/// - The process termination request fails.
/// - Spawning the user shell process fails.
pub fn restart_desktop_shell() -> Result<()> {
    let restart_strategy = determine_shell_restart_strategy();
    let shell_command = if restart_strategy == ShellRestartStrategy::ManualSpawnAsUser {
        Some(resolve_shell_command_line())
    } else {
        None
    };

    let shell_process_id = query_shell_process_id()?;
    terminate_process_by_id(shell_process_id)?;

    if let Some(command_line) = shell_command {
        spawn_shell_as_interactive_user(&command_line)?;
    }

    Ok(())
}

/// Determine whether the shell should be restarted by Winlogon or manually spawned.
fn determine_shell_restart_strategy() -> ShellRestartStrategy {
    let local_machine_root = RegKey::predef(HKEY_LOCAL_MACHINE);
    let winlogon_key =
        match local_machine_root.open_subkey_with_flags(WINLOGON_REGISTRY_PATH, KEY_QUERY_VALUE) {
            Ok(key) => key,
            Err(_) => return ShellRestartStrategy::ManualSpawnAsUser,
        };

    let auto_restart_value_result: Result<u32, _> =
        winlogon_key.get_value(AUTO_RESTART_SHELL_VALUE_NAME);
    match auto_restart_value_result {
        Ok(current_value) if current_value == AUTO_RESTART_SHELL_ENABLED_VALUE => {
            ShellRestartStrategy::AutomaticByWinlogon
        }
        _ => ShellRestartStrategy::ManualSpawnAsUser,
    }
}

/// Resolve the configured Shell command line, prioritizing HKCU over HKLM with fallback to `"explorer.exe"`.
fn resolve_shell_command_line() -> String {
    // 1. Try reading the interactive user's HKCU configuration
    if let Ok(user_root_key) = session::open_active_user_registry_root() {
        if let Ok(user_winlogon_key) =
            user_root_key.open_subkey_with_flags(WINLOGON_REGISTRY_PATH, KEY_QUERY_VALUE)
        {
            if let Ok(shell_value) =
                user_winlogon_key.get_value::<String, _>(SHELL_REGISTRY_VALUE_NAME)
            {
                let trimmed_shell_value = shell_value.trim();
                if !trimmed_shell_value.is_empty() {
                    return trimmed_shell_value.to_string();
                }
            }
        }
    }

    // 2. Try reading the system HKLM configuration
    let local_machine_root = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(system_winlogon_key) =
        local_machine_root.open_subkey_with_flags(WINLOGON_REGISTRY_PATH, KEY_QUERY_VALUE)
    {
        if let Ok(shell_value) =
            system_winlogon_key.get_value::<String, _>(SHELL_REGISTRY_VALUE_NAME)
        {
            let trimmed_shell_value = shell_value.trim();
            if !trimmed_shell_value.is_empty() {
                return trimmed_shell_value.to_string();
            }
        }
    }

    // 3. Fallback to hardcoded default
    DEFAULT_SHELL_COMMAND.to_string()
}

/// Spawn the specified shell command line under the active interactive user security context.
fn spawn_shell_as_interactive_user(command_line: &str) -> Result<()> {
    let session_user_token =
        session::fetch_active_user_token().context(t!("ERROR_QUERY_USER_TOKEN_FAILED"))?;

    let primary_user_token = session_user_token
        .duplicate(TokenType::Primary)
        .context(t!("ERROR_DUPLICATE_TOKEN_FAILED"))?;

    ProcessSpawner::new_with_token(&primary_user_token)
        .command_line(command_line)
        .desktop(INTERACTIVE_DESKTOP_PATH)
        .spawn()
        .context(t!("ERROR_SPAWN_SHELL_PROCESS_FAILED"))?;

    Ok(())
}

/// Query the process identifier owning the active desktop shell window.
fn query_shell_process_id() -> Result<u32> {
    let shell_window_hwnd = unsafe { GetShellWindow() };
    if shell_window_hwnd.0.is_null() {
        bail!("{}", t!("ERROR_DESKTOP_SHELL_WINDOW_NOT_FOUND"));
    }

    let mut shell_process_id = 0;
    let window_thread_id =
        unsafe { GetWindowThreadProcessId(shell_window_hwnd, Some(&mut shell_process_id)) };

    if window_thread_id == 0 || shell_process_id == 0 {
        bail!("{}", t!("ERROR_QUERY_SHELL_CLIENT_ID_FAILED"));
    }

    Ok(shell_process_id)
}

/// Terminate the target process using `PROCESS_TERMINATE` access rights.
fn terminate_process_by_id(process_id: u32) -> Result<()> {
    let process_handle = unsafe { OpenProcess(PROCESS_TERMINATE, false, process_id) }
        .with_context(|| {
            t!(
                "ERROR_OPEN_SHELL_PROCESS_FAILED",
                shell_process_id = process_id
            )
        })?;
    let process_guard = ProcessHandleGuard::new(process_handle);

    unsafe {
        TerminateProcess(process_guard.as_raw(), 0).with_context(|| {
            t!(
                "ERROR_TERMINATE_SHELL_PROCESS_FAILED",
                shell_process_id = process_id
            )
        })?;
    }

    Ok(())
}
