//! User interface modal dialogs and message box abstractions.
//!
//! # Module Structure
//! - [`message`] — implementations of information, confirmation, error, and fatal dialogs

mod message;

pub use message::{
    UserConfirmationOutcome, prompt_confirmation_dialog, show_error_dialog,
    show_fatal_error_dialog, show_info_dialog,
};
