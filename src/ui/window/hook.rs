//! UI customization hook for stripping Win32 visual themes.
//!
//! Installs a thread-local `WH_CALLWNDPROC` hook that intercepts `WM_CREATE`
//! for every window and control created on this thread. Three customizations
//! are applied at creation time:
//!
//! 1. **DWM transition animations disabled** — removes the fade-in effect
//!    when windows are shown, which can look jarring in a utility application.
//! 2. **Visual theme removed** — [`SetWindowTheme`] with empty strings opts the
//!    window out of the current visual style, giving controls a classic flat look.
//! 3. **Theme app properties cleared** — [`SetThemeAppProperties`] with flags `0`
//!    prevents any themed painting from occurring on this thread.
//!
//! The hook must be installed before any window is created so it can intercept
//! their `WM_CREATE` messages.

use std::{ffi::c_void, ptr};
use windows::{
    Win32::{
        Foundation::{BOOL, LPARAM, LRESULT, TRUE, WPARAM},
        Graphics::Dwm::{DWMWA_TRANSITIONS_FORCEDISABLED, DwmSetWindowAttribute},
        System::Threading::GetCurrentThreadId,
        UI::{
            Controls::{SET_THEME_APP_PROPERTIES_FLAGS, SetThemeAppProperties, SetWindowTheme},
            WindowsAndMessaging::{
                CWPSTRUCT, CallNextHookEx, SetWindowsHookExW, WH_CALLWNDPROC, WM_CREATE,
            },
        },
    },
    core::w,
};

/// The `WH_CALLWNDPROC` hook procedure.
///
/// Called by Win32 before every window procedure on this thread receives a message.
/// When `hook_code >= 0` and the message is `WM_CREATE`, the three UI customizations
/// are applied to the target window. All messages are then forwarded to the next
/// hook in the chain via [`CallNextHookEx`].
extern "system" fn ui_customization_hook_procedure(
    hook_code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if hook_code >= 0 {
        let call_window_procedure_struct = unsafe { &*(lparam.0 as *const CWPSTRUCT) };

        if call_window_procedure_struct.message == WM_CREATE {
            // Disable the DWM fade-in transition for this window.
            let _ = unsafe {
                DwmSetWindowAttribute(
                    call_window_procedure_struct.hwnd,
                    DWMWA_TRANSITIONS_FORCEDISABLED,
                    ptr::from_ref::<BOOL>(&TRUE).cast::<c_void>(),
                    size_of::<BOOL>() as u32,
                )
            };

            // Remove the visual theme so the control renders in the classic style.
            let _ = unsafe { SetWindowTheme(call_window_procedure_struct.hwnd, w!(""), w!("")) };

            // Clear theme app properties to prevent any themed painting on this thread.
            unsafe { SetThemeAppProperties(SET_THEME_APP_PROPERTIES_FLAGS(0)) };
        }
    }

    unsafe { CallNextHookEx(None, hook_code, wparam, lparam) }
}

/// Install the UI customization hook on the current thread.
///
/// Panics if [`SetWindowsHookExW`] fails, because the application cannot
/// function correctly without the hook (controls would render with visual
/// themes applied).
pub fn install_ui_customization_hook() {
    unsafe {
        SetWindowsHookExW(
            WH_CALLWNDPROC,
            Some(ui_customization_hook_procedure),
            None,
            GetCurrentThreadId(),
        )
        .expect("INSTALL_UI_CUSTOMIZATION_HOOK_FAILED");
    };
}
