//! Supported application languages and system locale resolution.

use serde::{Deserialize, Serialize};

use crate::ui::menu::state::{IDM_LANG_EN_US, IDM_LANG_ZH_CN};

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
    /// Return all supported application languages in their display order.
    pub const fn all() -> &'static [Self] {
        &[Self::EnUs, Self::ZhCn]
    }

    /// Resolve an [`AppLanguage`] from a BCP 47 locale string slice.
    ///
    /// Returns `None` if the locale string is not explicitly supported.
    pub fn from_locale_str(locale_str: &str) -> Option<Self> {
        match locale_str {
            "zh-CN" => Some(Self::ZhCn),
            "en-US" => Some(Self::EnUs),
            _ => None,
        }
    }

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

        Self::from_locale_str(&system_locale).unwrap_or(Self::EnUs)
    }

    /// Returns the BCP 47 locale string for this language.
    ///
    /// The returned value is suitable for passing directly to
    /// [`rust_i18n::set_locale`].
    pub const fn as_locale_str(&self) -> &'static str {
        match self {
            Self::ZhCn => "zh-CN",
            Self::EnUs => "en-US",
        }
    }

    /// Return the endonym native display name of the language for UI presentation.
    pub const fn native_display_name(&self) -> &'static str {
        match self {
            Self::ZhCn => "简体中文",
            Self::EnUs => "English",
        }
    }

    /// Return the associated Win32 menu command identifier for this language.
    pub const fn menu_command_id(&self) -> u16 {
        match self {
            Self::ZhCn => IDM_LANG_ZH_CN,
            Self::EnUs => IDM_LANG_EN_US,
        }
    }
}
