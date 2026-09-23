//! Windows desktop shell process manipulation utilities.
//!
//! Provides routines to terminate and restart core Windows desktop shell processes (Explorer.exe).

use anyhow::{Context, Result, bail};
use rust_i18n::t;
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::Threading::{OpenProcess, PROCESS_TERMINATE, TerminateProcess},
    UI::WindowsAndMessaging::{GetShellWindow, GetWindowThreadProcessId},
};
use winreg::{
    RegKey,
    enums::{HKEY_LOCAL_MACHINE, KEY_QUERY_VALUE, KEY_SET_VALUE},
};

/// Subkey path to the Winlogon configuration in HKLM.
const WINLOGON_REGISTRY_PATH: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon";

/// Value name for the automatic shell restart setting.
const AUTO_RESTART_SHELL_VALUE_NAME: &str = "AutoRestartShell";

/// Expected value indicating that Winlogon automatically restarts Explorer.exe.
const AUTO_RESTART_SHELL_ENABLED_VALUE: u32 = 1;

/// RAII wrapper for a process handle to ensure proper closure via [`CloseHandle`].
struct ProcessHandleGuard {
    process_handle: HANDLE,
}

impl ProcessHandleGuard {
    fn new(process_handle: HANDLE) -> Self {
        Self { process_handle }
    }

    fn as_raw(&self) -> HANDLE {
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

/// Restart the Windows desktop shell (Explorer.exe) matching [System Informer]'s logic.
///
/// # Operational Behavior
/// 1. Verifies that `AutoRestartShell` is enabled in `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon`.
///    If set to `0` or disabled, explicitly writes `1` to ensure Winlogon will relaunch the shell.
/// 2. Identifies the process owning the primary desktop shell window (`GetShellWindow`).
/// 3. Opens the target process with `PROCESS_TERMINATE` rights and terminates it.
/// 4. System Winlogon detects the exit and automatically relaunches the shell.
///
/// # References
/// - [System Informer Repository](https://github.com/winsiderss/systeminformer)
///
/// # Errors
/// Returns an error if:
/// - The `Winlogon` registry key cannot be accessed or `AutoRestartShell` cannot be set to 1.
/// - The primary desktop shell window cannot be found.
/// - The shell process ID cannot be queried.
/// - The target process cannot be opened with termination rights.
/// - The process termination request fails.
pub fn restart_desktop_shell() -> Result<()> {
    ensure_auto_restart_shell_enabled()?;

    let shell_process_id = query_shell_process_id()?;
    terminate_process_by_id(shell_process_id)?;
    Ok(())
}

/// Ensure `AutoRestartShell` is enabled in the system Winlogon registry.
///
/// If `AutoRestartShell` is absent, Windows defaults to auto-restarting the shell.
/// If explicitly present and not equal to `1`, forces it to `1` so Explorer restarts.
fn ensure_auto_restart_shell_enabled() -> Result<()> {
    let local_machine_root = RegKey::predef(HKEY_LOCAL_MACHINE);
    let winlogon_key = local_machine_root
        .open_subkey_with_flags(WINLOGON_REGISTRY_PATH, KEY_QUERY_VALUE | KEY_SET_VALUE)
        .with_context(|| {
            t!(
                "ERROR_OPEN_WINLOGON_KEY_FAILED",
                registry_path = WINLOGON_REGISTRY_PATH
            )
        })?;

    let auto_restart_value_result: Result<u32, _> =
        winlogon_key.get_value(AUTO_RESTART_SHELL_VALUE_NAME);

    match auto_restart_value_result {
        Ok(current_value) if current_value == AUTO_RESTART_SHELL_ENABLED_VALUE => Ok(()),
        Ok(_) | Err(_) => winlogon_key
            .set_value(
                AUTO_RESTART_SHELL_VALUE_NAME,
                &AUTO_RESTART_SHELL_ENABLED_VALUE,
            )
            .with_context(|| {
                t!(
                    "ERROR_SET_AUTORESTARTSHELL_FAILED",
                    registry_path = WINLOGON_REGISTRY_PATH
                )
            }),
    }
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
