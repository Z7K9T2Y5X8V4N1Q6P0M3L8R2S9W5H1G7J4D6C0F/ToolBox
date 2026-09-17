//! Application entry point.
//!
//! Enforces execution under the TrustedInstaller identity, manages single-instance
//! execution, and initializes the application window.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use elevate_ti::{ElevationStatus, check_elevation_status, relaunch_as_trusted_installer};
use rust_i18n::{i18n, t};
use winsafe::{HWND, co, prelude::Handle};

use app::instance::{self, SingleInstanceStatus};

i18n!("locales", fallback = "en-US");

mod app;
mod config;
mod error;
mod ui;

fn main() {
    // 1. Verify elevation status before locking any single-instance mutex.
    match check_elevation_status() {
        Ok(ElevationStatus::RequiresElevation) => {
            if let Err(elevation_error) = relaunch_as_trusted_installer() {
                HWND::NULL
                    .MessageBox(
                        &format!("Failed to elevate to TrustedInstaller: {elevation_error}"),
                        &t!("ERROR"),
                        co::MB::OK | co::MB::ICONERROR,
                    )
                    .ok();
            }
            // Exit the parent process so the newly spawned TrustedInstaller instance runs.
            return;
        }
        Ok(ElevationStatus::TrustedInstaller) => {
            // Already elevated; proceed.
        }
        Err(check_error) => {
            log::warn!("Could not evaluate TrustedInstaller token status: {check_error}");
        }
    }

    // 2. Single-instance enforcement.
    let _single_instance_mutex_guard = match instance::check_single_instance() {
        SingleInstanceStatus::Primary(single_instance_mutex_guard) => single_instance_mutex_guard,
        SingleInstanceStatus::AlreadyRunning => return,
        SingleInstanceStatus::CreationFailed(windows_error) => {
            HWND::NULL
                .MessageBox(
                    &windows_error.to_string(),
                    &t!("ERROR"),
                    co::MB::OK | co::MB::ICONERROR,
                )
                .ok();
            return;
        }
    };

    // 3. UI loop startup.
    if let Err(error) = app::MainWindow::create_and_run() {
        HWND::NULL
            .MessageBox(
                &error.to_string(),
                &t!("ERROR"),
                co::MB::OK | co::MB::ICONERROR,
            )
            .ok();
    }
}
