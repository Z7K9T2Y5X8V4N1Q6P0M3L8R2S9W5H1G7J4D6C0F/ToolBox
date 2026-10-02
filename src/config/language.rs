//! Supported application languages and system locale resolution.

use serde::{Deserialize, Serialize};

/// A supported display language for the application.
///
/// Serialized as a BCP 47 locale string (e.g. `"zh-CN"`) in the config file.
/// Default initialization detects the operating system language and falls back
/// to [`AppLanguage::EnUs`] if detection fails or does not strictly match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppLanguage {
    #[serde(rename = "zh-CN")]
    ZhCn,
    #[serde(rename = "en-US")]
    EnUs,
}

impl Default for AppLanguage {
    fn default() -> Self {
        Self::detect_from_system()
    }
}

impl AppLanguage {
    /// Detect the display language strictly from the operating system locale.
    ///
    /// Queries the current system locale via [`sys_locale::get_locale`].
    /// Resolves to [`AppLanguage::ZhCn`] only if the locale string is strictly
    /// `"zh-CN"`, and to [`AppLanguage::EnUs`] if it is strictly `"en-US"`.
    /// In all other cases (query failure, unsupported language codes, or partial
    /// variants), falls back to [`AppLanguage::EnUs`].
    pub fn detect_from_system() -> Self {
        let Some(system_locale) = sys_locale::get_locale() else {
            return Self::EnUs;
        };

        match system_locale.as_str() {
            "zh-CN" => Self::ZhCn,
            "en-US" => Self::EnUs,
            _ => Self::EnUs,
        }
    }

    /// Returns the BCP 47 locale string for this language.
    ///
    /// The returned value is suitable for passing directly to
    /// [`rust_i18n::set_locale`].
    pub fn as_locale_str(&self) -> &'static str {
        match self {
            AppLanguage::ZhCn => "zh-CN",
            AppLanguage::EnUs => "en-US",
        }
    }
}
