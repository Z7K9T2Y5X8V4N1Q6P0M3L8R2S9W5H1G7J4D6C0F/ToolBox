//! WinEvent Accessibility hook lifecycle manager.
//!
//! Provides RAII encapsulation for event hooks registered via `SetWinEventHook`,
//! guaranteeing unhooking on scope teardown.

use windows::Win32::UI::Accessibility::{
    HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent, WINEVENTPROC,
};

/// RAII wrapper for an active [`HWINEVENTHOOK`].
pub struct WinEventHookGuard {
    hook_handle: HWINEVENTHOOK,
}

impl Drop for WinEventHookGuard {
    fn drop(&mut self) {
        if !self.hook_handle.is_invalid() {
            unsafe {
                let _ = UnhookWinEvent(self.hook_handle);
            }
        }
    }
}

impl WinEventHookGuard {
    /// Register an out-of-context WinEvent hook over a given event range.
    pub fn install(
        event_min: u32,
        event_max: u32,
        event_procedure: WINEVENTPROC,
        process_id: u32,
        thread_id: u32,
        flags: u32,
    ) -> Option<Self> {
        let raw_hook_handle = unsafe {
            SetWinEventHook(
                event_min,
                event_max,
                None,
                event_procedure,
                process_id,
                thread_id,
                flags,
            )
        };

        if raw_hook_handle.is_invalid() {
            None
        } else {
            Some(Self {
                hook_handle: raw_hook_handle,
            })
        }
    }
}
