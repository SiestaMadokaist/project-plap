use domain::storage::StoragePrefix;
use pkg::{
    enums::stage::Stage,
    types::{strings::URL, time::Second},
    utils::{var_or, var_second},
};
use std::env;

pub struct DiffusionEnv {
    stage: String,
    civitai_host: String,
    sanity_run: String,

    pub localhost: bool,
    pub max_data_transfer: i64,
    pub watch_dir: String,
    // pub aws_region: String,
    pub discord_username: String,
    pub discord_webhook_url: URL,

    pub civitai_apikey: String,
    pub workdir: StoragePrefix,

    pub output_region: String,
    pub output_bucket: String,
    pub output_prefix: StoragePrefix,

    pub model_region: String,
    pub model_bucket: String,
    pub model_prefix: StoragePrefix,

    pub cache_ttl: Second,

    blacklist_tags: String,
}
const DEFAULT_MAX_DATA_TRANSFER_BYTES: i64 = 30 * 1024 * 1024 * 1024;

impl DiffusionEnv {
    pub fn from_env() -> Self {
        Self {
            localhost: env::var("IS_LOCALHOST").expect("IS_LOCALHOST flag must be set") == "true", // @todo
            stage: var_or("STAGE", "production"),
            sanity_run: var_or("SANITY_RUN", "false"),

            civitai_apikey: env::var("CIVITAI_APIKEY").expect("CIVITAI_APIKEY must be set"),
            civitai_host: env::var("CIVITAI_HOST").expect("CIVITAI_HOST must be set"),
            workdir: env::var("WORKDIR")
                .map(StoragePrefix)
                .expect("WORKDIR must be set"),

            watch_dir: env::var("WATCH_DIR").expect("WATCH_DIR must be set"),
            max_data_transfer: env::var("MAX_DATA_TRANSFER")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_MAX_DATA_TRANSFER_BYTES),
            // aws_region: var_or("AWS_REGION", "us-east-1"),
            discord_username: var_or("DISCORD_USERNAME", "diffusion-agent"),
            discord_webhook_url: env::var("DISCORD_WEBHOOK_URL")
                .map(URL)
                .expect("DISCORD_WEBHOOK_URL must be set"),

            model_region: env::var("MODEL_REGION").expect("MODEL REGION must not be empty"),
            model_bucket: env::var("MODEL_BUCKET").expect("MODEL BUCKET must not be empty"),
            model_prefix: env::var("MODEL_PREFIX")
                .map(StoragePrefix)
                .expect("MODEL PREFIX must be set"),

            output_region: var_or("OUTPUT_REGION", "ap-southeast-1"),
            output_bucket: env::var("OUTPUT_BUCKET").expect("OUTPUT_BUCKET must be set"),
            output_prefix: env::var("OUTPUT_PREFIX")
                .map(StoragePrefix)
                .expect("OUTPUT PREFIX must be set"),
            blacklist_tags: env::var("BLACKLIST_TAGS").expect("BLACKLIST_TAGS must be set"),
            cache_ttl: var_second("CACHE_TTL"),
        }
    }

    /// `BLACKLIST_TAGS` is a comma-separated list; split it, trim each entry, and
    /// drop blanks (trailing comma, `a,,b`, etc.).
    pub fn blacklist_tags(&self) -> Vec<String> {
        self.blacklist_tags
            .split(',')
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(String::from)
            .collect()
    }

    pub fn sanity_run(&self) -> bool {
        self.sanity_run == "true"
    }

    pub fn civitai_host(&self) -> URL {
        URL(self.civitai_host.clone())
    }

    pub fn stage(&self) -> Stage {
        self.stage.as_str().into()
    }
}
