#![windows_subsystem = "windows"]

use rust_i18n::{i18n, t};
use winsafe::prelude::Handle;

i18n!("locales", fallback = "en-US");

mod config;
mod core;
mod error;
mod ui;

fn main() {
    if let Err(error) = core::MainWindow::create_and_run() {
        winsafe::HWND::NULL
            .MessageBox(
                &error.to_string(),
                &t!("ERROR"),
                winsafe::co::MB::OK | winsafe::co::MB::ICONERROR,
            )
            .ok();
    }
}
