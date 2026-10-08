//! Global Windows Basic visual style monitoring engine.
//!
//! Spawns a dedicated background worker thread with its own Win32 message pump,
//! listens for `EVENT_SYSTEM_FOREGROUND` and `EVENT_OBJECT_SHOW` via `SetWinEventHook`,
//! and strips DWM non-client frame rendering on active windows using WinSafe's
//! `DwmSetWindowAttribute` followed by an immediate frame recalculation (`SWP_FRAMECHANGED`).
//!
//! # Architecture
//! - [`GlobalBasicStylesWatcher`]: Manages the lifecycle of the background thread via RAII.
//! - Background worker thread: Hooks foreground events and sweeps top-level windows via [`winsafe::EnumWindows`].

use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
};

use anyhow::{Context, Result};
use windows::Win32::{
    Foundation::{HWND as RawHwnd, LPARAM, WPARAM},
    UI::{
        Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent},
        WindowsAndMessaging::{
            DispatchMessageW, EVENT_OBJECT_SHOW, EVENT_SYSTEM_FOREGROUND, GA_ROOT, GWL_STYLE,
            GetAncestor, GetMessageW, GetWindowLongPtrW, MSG, OBJID_WINDOW, PostThreadMessageW,
            TranslateMessage, WINEVENT_OUTOFCONTEXT, WM_QUIT, WS_CHILD,
        },
    },
};
use winsafe::{
    DwmAttr::NcRenderingPolicy, EnumWindows, GetCurrentThreadId, HWND, HwndPlace, POINT, SIZE, co,
};

/// RAII wrapper for [`HWINEVENTHOOK`] ensuring it is cleanly unhooked on drop.
struct WinEventHookGuard {
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

/// The state of the global Basic styles watcher.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalBasicStylesState {
    /// Watcher is active and stripping DWM frame rendering.
    Enabled,
    /// Watcher is inactive.
    Disabled,
}

/// Controller managing the background global Basic styles watcher thread.
pub struct GlobalBasicStylesWatcher {
    is_running: Arc<AtomicBool>,
    worker_thread_handle: Option<JoinHandle<()>>,
    worker_thread_id: Option<u32>,
}

impl Default for GlobalBasicStylesWatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalBasicStylesWatcher {
    /// Create a new, unstarted global Basic styles watcher.
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            worker_thread_handle: None,
            worker_thread_id: None,
        }
    }

    /// Current operational state of the watcher.
    pub fn current_state(&self) -> GlobalBasicStylesState {
        if self.is_running.load(Ordering::SeqCst) {
            GlobalBasicStylesState::Enabled
        } else {
            GlobalBasicStylesState::Disabled
        }
    }

    /// Start the background global Basic styles watcher thread.
    ///
    /// Enumerates existing visible windows immediately to apply Basic styles,
    /// then registers the foreground and show event hooks to capture window activations.
    pub fn start(&mut self) -> Result<()> {
        if self.current_state() == GlobalBasicStylesState::Enabled {
            return Ok(());
        }

        self.is_running.store(true, Ordering::SeqCst);
        let thread_running_flag = Arc::clone(&self.is_running);

        let (thread_id_sender, thread_id_receiver) = std::sync::mpsc::channel();

        let worker_handle = thread::Builder::new()
            .name(
                "Z7K9T2Y5X8V4N1Q6P0M3L8R2S9W5H1G7J4D6C0F.Toolbox.GlobalBasicStylesWorker"
                    .to_string(),
            )
            .spawn(move || {
                run_watcher_worker_loop(thread_running_flag, thread_id_sender);
            })
            .context("Failed to spawn background basic styles worker thread")?;

        let worker_thread_id = thread_id_receiver
            .recv()
            .context("Failed to receive worker thread identifier")?;

        self.worker_thread_handle = Some(worker_handle);
        self.worker_thread_id = Some(worker_thread_id);

        Ok(())
    }

    /// Stop the background global Basic styles watcher thread and restore windows.
    ///
    /// Posts `WM_QUIT` to the worker thread, joins the handle, and enumerates all top-level
    /// windows to restore their default DWM window frame rendering policy.
    pub fn stop(&mut self) -> Result<()> {
        if self.current_state() == GlobalBasicStylesState::Disabled {
            return Ok(());
        }

        self.is_running.store(false, Ordering::SeqCst);

        if let Some(target_thread_id) = self.worker_thread_id.take() {
            unsafe {
                let _ = PostThreadMessageW(target_thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
            }
        }

        if let Some(worker_thread_handle) = self.worker_thread_handle.take() {
            let _ = worker_thread_handle.join();
        }

        restore_all_top_level_windows_dwm_policy();

        Ok(())
    }

    /// Toggle the running state between enabled and disabled.
    pub fn toggle(&mut self) -> Result<GlobalBasicStylesState> {
        match self.current_state() {
            GlobalBasicStylesState::Enabled => {
                self.stop()?;
                Ok(GlobalBasicStylesState::Disabled)
            }
            GlobalBasicStylesState::Disabled => {
                self.start()?;
                Ok(GlobalBasicStylesState::Enabled)
            }
        }
    }
}

impl Drop for GlobalBasicStylesWatcher {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

// ---------------------------------------------------------------------------
// Worker Thread Logic
// ---------------------------------------------------------------------------

/// Global storage holding the running flag accessed by the static hook procedure.
static GLOBAL_IS_WATCHER_ACTIVE: Mutex<Option<Arc<AtomicBool>>> = Mutex::new(None);

/// Main loop for the watcher thread.
fn run_watcher_worker_loop(
    thread_running_flag: Arc<AtomicBool>,
    thread_id_sender: std::sync::mpsc::Sender<u32>,
) {
    let current_worker_thread_id = GetCurrentThreadId();
    if thread_id_sender.send(current_worker_thread_id).is_err() {
        return;
    }

    {
        let mut watcher_active_mutex_guard = GLOBAL_IS_WATCHER_ACTIVE
            .lock()
            .unwrap_or_else(|watcher_mutex_poison_error| watcher_mutex_poison_error.into_inner());
        *watcher_active_mutex_guard = Some(Arc::clone(&thread_running_flag));
    }

    // Step 1: Pre-warm all existing visible top-level root windows to Basic style.
    apply_basic_style_to_all_top_level_windows();

    // Step 2: Hook both foreground changes and window shows to eliminate timing lags.
    let hook_raw_handle = unsafe {
        SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_OBJECT_SHOW,
            None,
            Some(foreground_window_event_callback),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        )
    };

    if hook_raw_handle.is_invalid() {
        thread_running_flag.store(false, Ordering::SeqCst);
        return;
    }

    let _hook_guard = WinEventHookGuard {
        hook_handle: hook_raw_handle,
    };

    // Step 3: Run the worker message loop to receive out-of-context event dispatches.
    let mut thread_message = MSG::default();
    while unsafe { GetMessageW(&mut thread_message, RawHwnd::default(), 0, 0).as_bool() } {
        let _ = unsafe { TranslateMessage(&thread_message) };
        unsafe { DispatchMessageW(&thread_message) };
    }

    if let Ok(mut global_flag_guard) = GLOBAL_IS_WATCHER_ACTIVE.lock() {
        *global_flag_guard = None;
    }
}

/// Out-of-context callback fired whenever a window gains focus or is presented on screen.
unsafe extern "system" fn foreground_window_event_callback(
    _event_hook: HWINEVENTHOOK,
    event_id: u32,
    window_raw_handle: RawHwnd,
    object_id: i32,
    child_id: i32,
    _event_thread_id: u32,
    _event_time_ms: u32,
) {
    let is_watcher_currently_active = GLOBAL_IS_WATCHER_ACTIVE
        .lock()
        .ok()
        .and_then(|watcher_active_mutex_guard| {
            watcher_active_mutex_guard
                .as_ref()
                .map(|watcher_running_atomic_flag| {
                    watcher_running_atomic_flag.load(Ordering::SeqCst)
                })
        })
        .unwrap_or(false);

    if !is_watcher_currently_active || window_raw_handle.0.is_null() {
        return;
    }

    // Only process window-level events, discarding any control-level object events.
    if object_id != OBJID_WINDOW.0 || child_id != 0 {
        return;
    }

    if event_id != EVENT_SYSTEM_FOREGROUND && event_id != EVENT_OBJECT_SHOW {
        return;
    }

    // Strictly ensure the handle is itself a true top-level root window (not an inner child container).
    let root_raw_handle = unsafe { GetAncestor(window_raw_handle, GA_ROOT) };
    if root_raw_handle.0.is_null() || root_raw_handle != window_raw_handle {
        return;
    }

    let window_style = unsafe { GetWindowLongPtrW(window_raw_handle, GWL_STYLE) as u32 };
    if (window_style & WS_CHILD.0) != 0 {
        return;
    }

    let target_window_hwnd = unsafe { HWND::from_ptr(window_raw_handle.0) };
    if target_window_hwnd.IsWindow() && target_window_hwnd.IsWindowVisible() {
        force_window_non_client_basic_style(&target_window_hwnd);
    }
}

// ---------------------------------------------------------------------------
// Non-Client Area Operations
// ---------------------------------------------------------------------------

/// Apply the Basic non-client rendering policy and force frame recalculation.
fn force_window_non_client_basic_style(target_window_hwnd: &HWND) {
    let _ = target_window_hwnd
        .DwmSetWindowAttribute(NcRenderingPolicy(co::DWMNCRENDERINGPOLICY::DISABLED));

    // Strictly notify non-client area to redraw without moving, sizing, or activating.
    let _ = target_window_hwnd.SetWindowPos(
        HwndPlace::None,
        POINT::default(),
        SIZE::default(),
        co::SWP::NOMOVE
            | co::SWP::NOSIZE
            | co::SWP::NOZORDER
            | co::SWP::NOACTIVATE
            | co::SWP::FRAMECHANGED,
    );
}

/// Restore default DWM non-client rendering policy.
fn restore_window_non_client_default_style(target_window_hwnd: &HWND) {
    let _ = target_window_hwnd
        .DwmSetWindowAttribute(NcRenderingPolicy(co::DWMNCRENDERINGPOLICY::USEWINDOWSTYLE));

    let _ = target_window_hwnd.SetWindowPos(
        HwndPlace::None,
        POINT::default(),
        SIZE::default(),
        co::SWP::NOMOVE
            | co::SWP::NOSIZE
            | co::SWP::NOZORDER
            | co::SWP::NOACTIVATE
            | co::SWP::FRAMECHANGED,
    );
}

/// Apply Basic non-client frame to all visible top-level windows.
fn apply_basic_style_to_all_top_level_windows() {
    let _ = EnumWindows(|target_hwnd| {
        if target_hwnd.IsWindow() && target_hwnd.IsWindowVisible() {
            let target_raw_handle = RawHwnd(target_hwnd.ptr());
            let root_raw_handle = unsafe { GetAncestor(target_raw_handle, GA_ROOT) };
            if root_raw_handle == target_raw_handle {
                let window_style =
                    unsafe { GetWindowLongPtrW(target_raw_handle, GWL_STYLE) as u32 };
                if (window_style & WS_CHILD.0) == 0 {
                    force_window_non_client_basic_style(&target_hwnd);
                }
            }
        }
        true
    });
}

/// Restore default DWM non-client frame on all visible top-level windows.
fn restore_all_top_level_windows_dwm_policy() {
    let _ = EnumWindows(|target_hwnd| {
        if target_hwnd.IsWindow() && target_hwnd.IsWindowVisible() {
            let target_raw_handle = RawHwnd(target_hwnd.ptr());
            let root_raw_handle = unsafe { GetAncestor(target_raw_handle, GA_ROOT) };
            if root_raw_handle == target_raw_handle {
                let window_style =
                    unsafe { GetWindowLongPtrW(target_raw_handle, GWL_STYLE) as u32 };
                if (window_style & WS_CHILD.0) == 0 {
                    restore_window_non_client_default_style(&target_hwnd);
                }
            }
        }
        true
    });
}
