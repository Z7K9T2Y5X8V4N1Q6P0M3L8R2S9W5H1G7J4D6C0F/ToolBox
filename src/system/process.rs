//! System process manipulation utilities.
//!
//! Provides routines to terminate and restart core Windows processes, matching
//! the implementation found in System Informer (Process Hacker).

use anyhow::{Context, Result, bail};
use rust_i18n::t;
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::Threading::{OpenProcess, PROCESS_TERMINATE, TerminateProcess},
    UI::WindowsAndMessaging::{GetShellWindow, GetWindowThreadProcessId},
};

/// RAII wrapper for a process handle to ensure proper closure via [`CloseHandle`].
struct ProcessHandleGuard {
    process_handle: HANDLE,
}

impl ProcessHandleGuard {
    /// Wrap a valid process handle.
    fn new(process_handle: HANDLE) -> Self {
        Self { process_handle }
    }

    /// Access the underlying raw process handle.
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

/// Restart the Windows desktop shell (Explorer.exe) matching [System Informer]'s logic
///
/// # Operational Behavior
/// Identifies the process owning the primary desktop shell window (`GetShellWindow`),
/// opens the target process with `PROCESS_TERMINATE` rights, and terminates it.
/// System Winlogon then detects the exit and automatically relaunches the shell.
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
pub fn restart_desktop_shell() -> Result<()> {
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

    let process_handle = unsafe { OpenProcess(PROCESS_TERMINATE, false, shell_process_id) }
        .with_context(|| {
            t!(
                "ERROR_OPEN_SHELL_PROCESS_FAILED",
                shell_process_id = shell_process_id
            )
        })?;
    let process_guard = ProcessHandleGuard::new(process_handle);

    unsafe {
        TerminateProcess(process_guard.as_raw(), 0).with_context(|| {
            t!(
                "ERROR_TERMINATE_SHELL_PROCESS_FAILED",
                shell_process_id = shell_process_id
            )
        })?;
    }

    Ok(())
}
