use crate::{
    config::helper::{var_or, var_second},
    pkg::{enums::stage::Stage, types::time::Second},
};
use std::env;

pub struct DiffusionEnv {
    stage: String,
    pub localhost: bool,
    pub max_data_transfer: i64,
    pub watch_dir: String,
    pub aws_region: String,
    pub discord_webhook_url: String,
    pub output_bucket: String,
    pub output_prefix: String,
    pub watch_interval: Second,
    pub queue_interval: Second,
    pub idle_tolerance: Second,
}
const DEFAULT_MAX_DATA_TRANSFER_BYTES: i64 = 30 * 1024 * 1024 * 1024;

impl DiffusionEnv {
    pub fn from_env() -> Self {
        Self {
            localhost: true, // @todo
            stage: var_or("STAGE", "production"),
            watch_dir: env::var("WATCH_DIR").expect("WATCH_DIR must be set"),
            max_data_transfer: env::var("MAX_DATA_TRANSFER")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_MAX_DATA_TRANSFER_BYTES),
            aws_region: var_or("AWS_REGION", "us-east-1"),
            discord_webhook_url: env::var("DISCORD_WEBHOOK_URL")
                .expect("DISCORD_WEBHOOK_URL must be set"),
            output_bucket: env::var("OUTPUT_BUCKET").expect("OUTPUT_BUCKET must be set"),
            output_prefix: var_or("OUTPUT_PREFIX", ""),
            watch_interval: var_second("WATCH_INTERVAL"),
            queue_interval: var_second("QUEUE_INTERVAL"),
            idle_tolerance: var_second("IDLE_TOLERANCE"),
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
