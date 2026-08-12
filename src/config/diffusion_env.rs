use crate::{config::helper::var_or, pkg::enums::stage::Stage};
use std::env;

pub struct DiffusionEnv {
    stage: String,
    pub max_data_transfer: i64,
    pub watch_dir: String,
    pub aws_region: String,
    pub discord_webhook_url: String,
    pub output_bucket: String,
    pub output_prefix: String,
}
const DEFAULT_MAX_DATA_TRANSFER_BYTES: i64 = 30 * 1024 * 1024 * 1024;

impl DiffusionEnv {
    pub fn from_env() -> Self {
        Self {
            stage: var_or("STAGE", "production"),
            watch_dir: var_or("WATCH_DIR", "./"),
            max_data_transfer: env::var("MAX_DATA_TRANSFER")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_MAX_DATA_TRANSFER_BYTES),
            aws_region: var_or("AWS_REGION", "us-east-1"),
            discord_webhook_url: env::var("DISCORD_WEBHOOK_URL")
                .expect("DISCORD_WEBHOOK_URL must be set"),
            output_bucket: env::var("OUTPUT_BUCKET").expect("OUTPUT_BUCKET must be set"),
            output_prefix: var_or("OUTPUT_PREFIX", ""),
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
