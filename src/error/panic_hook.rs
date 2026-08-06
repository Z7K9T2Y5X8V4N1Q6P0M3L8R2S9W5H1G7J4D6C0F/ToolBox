use rust_i18n::t;
use std::panic;
use winsafe::{HWND, co, prelude::Handle};

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

fn extract_panic_message(payload: &dyn std::any::Any) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        return message.to_string();
    }
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    t!("FATAL_ERROR_UNKNOWN_ERROR").to_string()
}
