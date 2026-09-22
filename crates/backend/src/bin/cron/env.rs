use std::env;

use domain::storage::StoragePrefix;
use pkg::{enums::stage::Stage, types::strings::URL, utils::var_or};
pub struct CronEnv {
    stage: String,
    pub sentry_dsn: Option<String>,
    pub sentry_env: String,
    pub openai_model: String,
    pub syosetu_host: String,
    pub proxy_host: String,
    pub proxy_port: String,
    pub proxy_username: String,
    pub proxy_password: String,
    pub tl_region: String,
    pub tl_bucket: String,
    pub tl_rootdir: StoragePrefix,
    pub discord_username: String,
    pub discord_webhook_url: URL,
    pub max_data_transfer: i64,
}

const LAMBDA_MAX_DATA_TRANSFER_BYTES: i64 = 2 * 1024 * 1024 * 1024;

impl CronEnv {
    pub fn from_env() -> Self {
        Self {
            stage: var_or("STAGE", "production"),
            sentry_dsn: env::var("SENTRY_DSN").ok().filter(|s| !s.is_empty()),
            sentry_env: var_or("SENTRY_ENV", "unknown"),
            openai_model: var_or("OPENAI_MODEL", "gpt-4o"),
            syosetu_host: var_or("SYOSETU_HOST", "https://ncode.syosetu.com"),
            proxy_host: env::var("PROXY_HOST").expect("PROXY_HOST must be set"),
            proxy_port: env::var("PROXY_PORT").expect("PROXY_PORT must be set"),
            proxy_username: env::var("PROXY_USERNAME").expect("PROXY_USERNAME must be set"),
            proxy_password: env::var("PROXY_PASSWORD").expect("PROXY_PASSWORD must be set"),
            tl_region: env::var("TL_REGION").expect("TL_REGION must be set"),
            tl_bucket: env::var("TL_BUCKET").expect("TL_BUCKET must be set"),
            tl_rootdir: env::var("TL_ROOTDIR")
                .map(StoragePrefix)
                .expect("TL_ROOTDIR must be set"),
            discord_username: var_or("DISCORD_USERNAME", "lambda-cron"),
            discord_webhook_url: env::var("DISCORD_WEBHOOK_URL")
                .map(URL)
                .expect("DISCORD_WEBHOOK_URL must be set"),
            max_data_transfer: env::var("MAX_DATA_TRANSFER")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(LAMBDA_MAX_DATA_TRANSFER_BYTES),
        }
    }

    pub fn stage(&self) -> Stage {
        match self.stage.as_str() {
            "development" => Stage::Development,
            "staging" => Stage::Staging,
            "production" => Stage::Production,
            _ => Stage::Development,
        }
    }
}
