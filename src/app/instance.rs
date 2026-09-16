//! Single instance process synchronization.
//!
//! Ensures that only one instance of the application runs at any given time.
//! When a secondary instance attempts to start, it signals the primary instance
//! via a registered window broadcast message to restore its window and bring it
//! to the foreground, then terminates immediately.

use windows::{
    Win32::{
        Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE, LPARAM, WPARAM},
        System::Threading::CreateMutexW,
        UI::WindowsAndMessaging::{HWND_BROADCAST, PostMessageW, RegisterWindowMessageW},
    },
    core::{Error as WindowsError, PCWSTR, w},
};

/// Unique system-wide name for the application instance mutex.
const INSTANCE_MUTEX_NAME: PCWSTR =
    w!("Local\\Z7K9T2Y5X8V4N1Q6P0M3L8R2S9W5H1G7J4D6C0F.Toolbox.SingleInstanceMutex");

/// Registered message name for waking up the existing primary instance window.
const RESTORE_WINDOW_MESSAGE_NAME: PCWSTR =
    w!("Z7K9T2Y5X8V4N1Q6P0M3L8R2S9W5H1G7J4D6C0F.Toolbox.RestoreExistingWindowMessage");

/// The outcome of attempting to acquire the application single-instance mutex.
pub enum SingleInstanceStatus {
    /// This is the primary instance; ownership of the mutex guard is granted.
    Primary(SingleInstanceGuard),
    /// Another instance is already running; the primary instance has been signaled.
    AlreadyRunning,
    /// Failed to create or inspect the mutex due to a Windows system error.
    CreationFailed(WindowsError),
}

/// RAII guard holding the named mutex handle alive for the lifetime of the application.
pub struct SingleInstanceGuard {
    mutex_handle: HANDLE,
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        if !self.mutex_handle.is_invalid() {
            unsafe {
                let _ = CloseHandle(self.mutex_handle);
            }
        }
    }
}

/// Attempt to obtain primary application instance status.
///
/// Returns:
/// - [`SingleInstanceStatus::Primary`] if this is the first instance.
/// - [`SingleInstanceStatus::AlreadyRunning`] if an existing instance was detected and notified.
/// - [`SingleInstanceStatus::CreationFailed`] if the Win32 API call failed.
pub fn check_single_instance() -> SingleInstanceStatus {
    let mutex_creation_result = unsafe { CreateMutexW(None, false, INSTANCE_MUTEX_NAME) };

    let mutex_handle = match mutex_creation_result {
        Ok(mutex_handle) => mutex_handle,
        Err(windows_error) => return SingleInstanceStatus::CreationFailed(windows_error),
    };

    let is_already_exists = unsafe { GetLastError() == ERROR_ALREADY_EXISTS };
    if is_already_exists {
        unsafe {
            let _ = CloseHandle(mutex_handle);
        }
        signal_existing_instance();
        SingleInstanceStatus::AlreadyRunning
    } else {
        let single_instance_mutex_guard = SingleInstanceGuard { mutex_handle };
        SingleInstanceStatus::Primary(single_instance_mutex_guard)
    }
}

/// Retrieve the unique registered Win32 message identifier used for restoring the primary window.
pub fn get_restore_window_message_id() -> u32 {
    unsafe { RegisterWindowMessageW(RESTORE_WINDOW_MESSAGE_NAME) }
}

/// Broadcast the wake-up message across all top-level windows to notify the primary instance.
fn signal_existing_instance() {
    let restore_message_id = get_restore_window_message_id();
    if restore_message_id != 0 {
        unsafe {
            let _ = PostMessageW(HWND_BROADCAST, restore_message_id, WPARAM(0), LPARAM(0));
        }
    }
}
