//! Implementation of system message box dialogs.
//!
//! Provides routines for displaying modal information, confirmation,
//! error, and fatal crash message boxes with optional parent window binding.

use rust_i18n::t;
use winsafe::{AnyResult, HWND, co, prelude::Handle};

/// The user's response to an interactive confirmation dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserConfirmationOutcome {
    /// The user confirmed the action (clicked "Yes").
    Confirmed,
    /// The user declined or cancelled the action (clicked "No" or closed dialog).
    Cancelled,
}

/// Display a modal information message box with an "OK" button.
///
/// When `owner_window_handle` is provided, the dialog centers on and disables
/// the specified parent window. Otherwise, it is displayed as a task-modal ownerless dialog.
pub fn show_info_dialog(
    owner_window_handle: Option<&HWND>,
    title: &str,
    message: &str,
) -> AnyResult<()> {
    let resolved_owner_handle = owner_window_handle.unwrap_or(&HWND::NULL);
    let mut dialog_style_flags = co::MB::OK | co::MB::ICONINFORMATION;

    if owner_window_handle.is_none() {
        dialog_style_flags |= co::MB::TASKMODAL;
    }

    resolved_owner_handle.MessageBox(message, title, dialog_style_flags)?;
    Ok(())
}

/// Display a modal confirmation message box with "Yes" and "No" choices.
///
/// When `owner_window_handle` is provided, the dialog centers on and disables
/// the specified parent window until answered.
pub fn prompt_confirmation_dialog(
    owner_window_handle: Option<&HWND>,
    title: &str,
    prompt_message: &str,
) -> AnyResult<UserConfirmationOutcome> {
    let resolved_owner_handle = owner_window_handle.unwrap_or(&HWND::NULL);
    let mut dialog_style_flags = co::MB::YESNO | co::MB::ICONQUESTION;

    if owner_window_handle.is_none() {
        dialog_style_flags |= co::MB::TASKMODAL;
    }

    let dialog_result =
        resolved_owner_handle.MessageBox(prompt_message, title, dialog_style_flags)?;

    if dialog_result == co::DLGID::YES {
        Ok(UserConfirmationOutcome::Confirmed)
    } else {
        Ok(UserConfirmationOutcome::Cancelled)
    }
}

/// Display a modal error dialog.
///
/// When `owner_window_handle` is provided, the dialog centers on and disables
/// the specified parent window. Otherwise, it defaults to a task-modal ownerless dialog.
pub fn show_error_dialog(owner_window_handle: Option<&HWND>, error_message: &str) {
    let resolved_owner_handle = owner_window_handle.unwrap_or(&HWND::NULL);
    let mut dialog_style_flags = co::MB::OK | co::MB::ICONERROR;

    if owner_window_handle.is_none() {
        dialog_style_flags |= co::MB::TASKMODAL;
    }

    resolved_owner_handle
        .MessageBox(error_message, &t!("ERROR"), dialog_style_flags)
        .ok();
}

/// Display an ownerless, blocking fatal error dialog during unhandled crashes.
///
/// Binds directly to [`HWND::NULL`] and forces [`co::MB::TASKMODAL`] so it remains
/// visible and blocks input even when the main window is corrupted or nonexistent.
pub fn show_fatal_error_dialog(fatal_error_message: &str) {
    HWND::NULL
        .MessageBox(
            fatal_error_message,
            &t!("FATAL_ERROR_TITLE"),
            co::MB::OK | co::MB::ICONERROR | co::MB::TASKMODAL,
        )
        .ok();
}
