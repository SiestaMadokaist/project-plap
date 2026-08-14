use std::env;

use crate::{config::helper::var_or, pkg::enums::stage::Stage};
pub struct CronEnv {
    stage: String,
    pub translation_table: String,
    pub openai_model: String,
    pub syosetu_host: String,
    pub proxy_host: Option<String>,
    pub proxy_port: Option<String>,
    pub proxy_username: Option<String>,
    pub proxy_password: Option<String>,
    pub tl_region: String,
    pub tl_bucket: String,
    pub tl_prefix: String,
    pub discord_webhook_url: String,
    pub max_data_transfer: i64,
}

const LAMBDA_MAX_DATA_TRANSFER_BYTES: i64 = 1 * 1024 * 1024 * 1024;

impl CronEnv {
    pub fn from_env() -> Self {
        Self {
            stage: var_or("STAGE", "production"),
            translation_table: var_or("TRANSLATION_TABLE", "production-translations"),
            openai_model: var_or("OPENAI_MODEL", "gpt-4o"),
            syosetu_host: var_or("SYOSETU_HOST", "https://ncode.syosetu.com"),
            proxy_host: env::var("PROXY_HOST").ok(),
            proxy_port: env::var("PROXY_PORT").ok(),
            proxy_username: env::var("PROXY_USERNAME").ok(),
            proxy_password: env::var("PROXY_PASSWORD").ok(),
            tl_region: env::var("TL_REGION").expect("TL_REGION must be set"),
            tl_bucket: env::var("TL_BUCKET").expect("TL_BUCKET must be set"),
            tl_prefix: var_or("TL_PREFIX", ""),
            discord_webhook_url: env::var("DISCORD_WEBHOOK_URL")
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
            "production" => Stage::Staging,
            _ => Stage::Development,
        }
    }
}
