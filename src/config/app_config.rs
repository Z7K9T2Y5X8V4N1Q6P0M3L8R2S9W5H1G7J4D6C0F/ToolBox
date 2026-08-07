use anyhow::{Context, Result};
use rust_i18n::t;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use winsafe::prelude::Handle;

use crate::config::AppLanguage;
use crate::config::ConfigLoadResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct AppConfig {
    pub language: AppLanguage,
    pub disclaimer_accepted: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: AppLanguage::default(),
            disclaimer_accepted: false,
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
        match Self::try_load() {
            ConfigLoadResult::Loaded(parsed_config) => parsed_config,
            ConfigLoadResult::NotFound(default_config) => {
                if let Err(save_error) = default_config.save() {
                    winsafe::HWND::NULL
                        .MessageBox(
                            &t!("CONFIG_SAVE_DEFAULT_FAILED", save_error = save_error),
                            &t!("ERROR"),
                            winsafe::co::MB::OK | winsafe::co::MB::ICONERROR,
                        )
                        .ok();
                }
                default_config
            }
            ConfigLoadResult::ParseFailed(parse_error) => {
                winsafe::HWND::NULL
                    .MessageBox(
                        &t!(
                            "CONFIG_PARSE_FAILED_USING_DEFAULT",
                            parse_error = parse_error
                        ),
                        &t!("ERROR"),
                        winsafe::co::MB::OK | winsafe::co::MB::ICONERROR,
                    )
                    .ok();
                let default_config = Self::default();
                if let Err(save_error) = default_config.save() {
                    panic!(
                        "{}",
                        t!("CONFIG_SAVE_DEFAULT_FAILED", save_error = save_error)
                    );
                }
                default_config
            }
        }
    }

    fn try_load() -> ConfigLoadResult {
        let config_path = match Self::config_path() {
            Some(config_path) => config_path,
            None => return ConfigLoadResult::NotFound(Self::default()),
        };
        if !config_path.exists() {
            return ConfigLoadResult::NotFound(Self::default());
        }

        let file_content = match fs::read_to_string(&config_path).with_context(|| {
            t!(
                "CONFIG_READ_FAILED",
                config_path = config_path.display().to_string()
            )
        }) {
            Ok(file_content) => file_content,
            Err(read_error) => return ConfigLoadResult::ParseFailed(read_error),
        };

        match toml::from_str::<AppConfig>(&file_content).context(t!("CONFIG_PARSE_FAILED")) {
            Ok(parsed_config) => ConfigLoadResult::Loaded(parsed_config),
            Err(parse_error) => ConfigLoadResult::ParseFailed(parse_error),
        }
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

        let serialized_content =
            toml::to_string_pretty(self).context(t!("CONFIG_SERIALIZE_FAILED"))?;

        fs::write(&config_path, serialized_content).with_context(|| {
            t!(
                "CONFIG_WRITE_FAILED",
                config_path = config_path.display().to_string()
            )
        })?;

        Ok(())
    }
}
