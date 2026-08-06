use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::config::ConfigLoadResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct AppConfig {
    pub language: String,
    pub disclaimer_accepted: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: "en-US".to_string(),
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
                    eprintln!("默认配置保存失败: {save_error}");
                }
                default_config
            }
            ConfigLoadResult::ParseFailed(parse_error) => {
                eprintln!("配置文件解析失败，使用默认配置: {parse_error}");
                let default_config = Self::default();
                if let Err(save_error) = default_config.save() {
                    eprintln!("默认配置保存失败: {save_error}");
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

        let file_content = match fs::read_to_string(&config_path)
            .with_context(|| format!("无法读取配置文件: {}", config_path.display()))
        {
            Ok(file_content) => file_content,
            Err(read_error) => return ConfigLoadResult::ParseFailed(read_error),
        };

        match toml::from_str::<AppConfig>(&file_content).context("配置文件解析失败") {
            Ok(parsed_config) => ConfigLoadResult::Loaded(parsed_config),
            Err(parse_error) => ConfigLoadResult::ParseFailed(parse_error),
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path().context("无法获取配置目录")?;

        if let Some(config_path_parent) = config_path.parent() {
            fs::create_dir_all(config_path_parent)
                .with_context(|| format!("无法创建配置目录: {}", config_path_parent.display()))?;
        }

        let serialized_content = toml::to_string_pretty(self).context("配置序列化失败")?;

        fs::write(&config_path, serialized_content)
            .with_context(|| format!("无法写入配置文件: {}", config_path.display()))?;

        Ok(())
    }
}
