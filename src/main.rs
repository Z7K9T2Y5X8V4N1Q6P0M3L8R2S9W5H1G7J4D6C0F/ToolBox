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
mod desktop;
mod error;
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

    if let Err(app_runtime_error) = app::MainWindow::create_and_run() {
        let app_runtime_error_message = app_runtime_error.to_string();
        error::show_error_dialog(&app_runtime_error_message);
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
                let elevation_error_message = elevation_error.to_string();
                error::show_error_dialog(&t!(
                    "ERROR_ELEVATE_TO_TRUSTEDINSTALLER_FAILED",
                    error_message = elevation_error_message
                ));
            }
            ElevationOutcome::Terminate
        }
        Err(check_elevation_error) => {
            let check_elevation_error_message = check_elevation_error.to_string();
            error::show_error_dialog(&t!(
                "ERROR_EVALUATE_TRUSTEDINSTALLER_STATUS_FAILED",
                error_message = check_elevation_error_message
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
        SingleInstanceStatus::CreationFailed(create_mutex_error) => {
            let create_mutex_error_message = create_mutex_error.to_string();
            show_error_dialog(&create_mutex_error_message);
            None
        }
    }
}
