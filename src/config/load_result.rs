use super::app_config::AppConfig;

pub enum ConfigLoadResult {
    Loaded(AppConfig),
    NotFound(AppConfig),
    ParseFailed(anyhow::Error),
}
