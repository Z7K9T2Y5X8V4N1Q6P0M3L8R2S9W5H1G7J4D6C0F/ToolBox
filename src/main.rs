//! Application entry point.
//!
//! Initializes the i18n system and starts the main window. Any unhandled
//! error that propagates out of the message loop is shown in an error dialog
//! rather than printed to stderr.
//!
//! In release builds (`#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`),
//! the application runs without a console window. In debug builds, a console
//! is attached so panic messages and debug output remain visible.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rust_i18n::{i18n, t};
use winsafe::{HWND, co, prelude::Handle};

use app::instance::{self, SingleInstanceStatus};

i18n!("locales", fallback = "en-US");

mod app;
mod config;
mod error;
mod ui;

fn main() {
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
