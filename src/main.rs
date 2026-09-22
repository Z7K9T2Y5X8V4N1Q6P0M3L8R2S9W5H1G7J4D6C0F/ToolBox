//! Application entry point.
//!
//! Enforces execution under the TrustedInstaller identity, manages single-instance
//! execution, and initializes the application window.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use elevate_ti::ElevationStatus;
use rust_i18n::{i18n, t};

use app::{
    init,
    instance::{self, SingleInstanceGuard, SingleInstanceStatus},
};
use error::show_error_dialog;

i18n!("locales", fallback = "en-US");

mod app;
mod config;
mod error;
mod system;
mod ui;

/// The outcome of verifying process elevation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ElevationOutcome {
    /// Elevation criteria met; startup can safely continue.
    Proceed,
    /// Relaunched as elevated child or encountered a fatal error; terminate current process.
    Terminate,
}

fn main() {
    init::initialize_application();

    if ensure_trustedinstaller() == ElevationOutcome::Terminate {
        return;
    }

    let Some(_single_instance_guard) = acquire_single_instance() else {
        return;
    };

    if let Err(runtime_error) = app::MainWindow::create_and_run() {
        error::show_error_dialog(&runtime_error.to_string());
    }
}

/// Ensure the application is executing under the TrustedInstaller identity.
///
/// Handles automatic elevation relaunch or displays an error dialog if checks fail.
fn ensure_trustedinstaller() -> ElevationOutcome {
    match elevate_ti::check_elevation_status() {
        Ok(ElevationStatus::TrustedInstaller) => ElevationOutcome::Proceed,
        Ok(ElevationStatus::RequiresElevation) => {
            if let Err(elevation_error) = elevate_ti::relaunch_as_trustedinstaller() {
                error::show_error_dialog(&t!(
                    "ERROR_ELEVATE_TO_TRUSTEDINSTALLER_FAILED",
                    elevation_error = elevation_error
                ));
            }
            ElevationOutcome::Terminate
        }
        Err(check_error) => {
            error::show_error_dialog(&t!(
                "ERROR_EVALUATE_TRUSTEDINSTALLER_STATUS_FAILED",
                check_error = check_error
            ));
            ElevationOutcome::Terminate
        }
    }
}

/// Attempt to acquire the system-wide single instance mutex guard.
///
/// If another instance is running or mutex creation fails, displays an error
/// dialog (if applicable) and returns `None`.
fn acquire_single_instance() -> Option<SingleInstanceGuard> {
    match instance::check_single_instance() {
        SingleInstanceStatus::Primary(guard) => Some(guard),
        SingleInstanceStatus::AlreadyRunning => None,
        SingleInstanceStatus::CreationFailed(windows_error) => {
            show_error_dialog(&windows_error.to_string());
            None
        }
    }
}
