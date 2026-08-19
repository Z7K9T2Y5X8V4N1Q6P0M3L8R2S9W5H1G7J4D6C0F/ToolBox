use anyhow::{Context, Result};
use rust_i18n::t;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use winsafe::prelude::Handle;

use crate::config::AppLanguage;
use crate::config::ConfigLoadResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE", deny_unknown_fields)]
pub struct AppConfig {
    pub language: AppLanguage,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: AppLanguage::default(),
        }
    }
}

impl AppConfig {
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|base_config_directory| {
            base_config_directory
                .join(env!("CARGO_PKG_NAME"))
                .join("CONFIG.toml")
        })
    }

    pub fn load() -> Self {
        match Self::load_config_content() {
            ConfigLoadResult::Loaded(config) => config,
            ConfigLoadResult::NotFound(default_config) => {
                Self::handle_missing_config(default_config)
            }
            ConfigLoadResult::ParseFailed(error) => Self::handle_corrupted_config(error),
        }
    }

    fn load_config_content() -> ConfigLoadResult {
        let config_path = match Self::config_path() {
            Some(config_path) => config_path,
            None => return ConfigLoadResult::NotFound(Self::default()),
        };

        if !config_path.exists() {
            return ConfigLoadResult::NotFound(Self::default());
        }

        let toml_content = match Self::read_config_file(&config_path) {
            Ok(content) => content,
            Err(read_error) => return ConfigLoadResult::ParseFailed(read_error),
        };

        match Self::parse_config(&toml_content) {
            Ok(config) => ConfigLoadResult::Loaded(config),
            Err(parse_error) => ConfigLoadResult::ParseFailed(parse_error),
        }
    }

    fn read_config_file(config_path: &PathBuf) -> Result<String> {
        fs::read_to_string(config_path).with_context(|| {
            t!(
                "CONFIG_READ_FAILED",
                config_path = config_path.display().to_string()
            )
        })
    }

    fn parse_config(toml_content: &str) -> Result<AppConfig> {
        toml::from_str::<AppConfig>(toml_content).context(t!("CONFIG_PARSE_FAILED"))
    }

    fn handle_missing_config(default_config: Self) -> Self {
        if let Err(save_error) = default_config.save() {
            panic!(
                "{}",
                t!("CONFIG_SAVE_DEFAULT_FAILED", save_error = save_error)
            );
        }
        default_config
    }

    fn handle_corrupted_config(error: anyhow::Error) -> Self {
        Self::show_error_dialog(&t!(
            "CONFIG_PARSE_FAILED_USING_DEFAULT",
            parse_error = error
        ));

        let default_config = Self::default();
        if let Err(save_error) = default_config.save() {
            panic!(
                "{}",
                t!("CONFIG_SAVE_DEFAULT_FAILED", save_error = save_error)
            );
        }
        default_config
    }

    fn show_error_dialog(message: &str) {
        winsafe::HWND::NULL
            .MessageBox(
                message,
                &t!("ERROR"),
                winsafe::co::MB::OK | winsafe::co::MB::ICONERROR,
            )
            .ok();
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path().context(t!("CONFIG_DIR_NOT_FOUND"))?;

        if let Some(config_path_parent) = config_path.parent() {
            fs::create_dir_all(config_path_parent).with_context(|| {
                t!(
                    "CONFIG_DIR_CREATE_FAILED",
                    config_path_parent = config_path_parent.display().to_string()
                )
            })?;
        }

        let toml_content = toml::to_string_pretty(self).context(t!("CONFIG_SERIALIZE_FAILED"))?;
        fs::write(&config_path, toml_content).with_context(|| {
            t!(
                "CONFIG_WRITE_FAILED",
                config_path = config_path.display().to_string()
            )
        })?;

        Ok(())
    }
}
