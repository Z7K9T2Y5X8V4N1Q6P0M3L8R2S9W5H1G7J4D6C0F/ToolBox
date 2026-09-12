//! Custom panic hook for GUI applications.
//!
//! In a `#![windows_subsystem = "windows"]` application there is no console,
//! so the default panic handler's output to stderr is silently discarded.
//! This module replaces it with a modal error dialog so panics are always
//! visible to the user.

use rust_i18n::t;
use std::panic;
use winsafe::{HWND, co, prelude::Handle};

/// Install a panic hook that displays the panic message in a Win32 error dialog.
///
/// Must be called once during application initialization, before any code
/// that could panic. Replaces the default stderr-based panic handler.
pub fn install_panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        HWND::NULL
            .MessageBox(
                &t!(
                    "FATAL_ERROR_CONTENT",
                    error = extract_panic_message(panic_info.payload())
                ),
                &t!("FATAL_ERROR_TITLE"),
                co::MB::OK | co::MB::ICONERROR,
            )
            .ok();
    }));
}

/// Extract a human-readable message from a panic payload.
///
/// Rust panics can carry either a `&str` or a `String` as their payload.
/// If neither type matches (e.g. a panic with a custom payload type),
/// a generic fallback message is returned.
fn extract_panic_message(payload: &dyn std::any::Any) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        return message.to_string();
    }
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    t!("FATAL_ERROR_UNKNOWN_ERROR").to_string()
}
