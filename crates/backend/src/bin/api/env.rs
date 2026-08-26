use pkg::{enums::stage::Stage, types::strings::URL, utils::var_or};
use std::env;

pub struct ApiEnv {
    stage: String,
    sanity_run: String,

    pub discord_username: String,
    pub discord_webhook_url: URL,

    pub output_region: String,
    pub output_bucket: String,
    pub output_prefix: String,

    pub model_region: String,
    pub model_bucket: String,
    pub model_prefix: String,
}

impl ApiEnv {
    pub fn from_env() -> Self {
        Self {
            sanity_run: var_or("SANITY_RUN", "false"),
            stage: env::var("STAGE").expect("stage must be set"),
            discord_username: var_or("DISCORD_USERNAME", "lambda-api"),
            discord_webhook_url: env::var("DISCORD_WEBHOOK_URL")
                .map(URL)
                .expect("DISCORD_WEBHOOK_URL must be set"),

            output_region: var_or("OUTPUT_REGION", "ap-southeast-1"),
            output_bucket: env::var("OUTPUT_BUCKET").expect("OUTPUT_BUCKET must be set"),
            output_prefix: var_or("OUTPUT_PREFIX", ""),

            model_region: env::var("MODEL_REGION").expect("MODEL_REGION must be set"),
            model_bucket: env::var("MODEL_BUCKET").expect("MODEL_BUCKET must be set"),
            model_prefix: var_or("MODEL_PREFIX", ""),
        }
    }

    pub fn sanity_run(&self) -> bool {
        self.sanity_run == "true"
    }

    pub fn stage(&self) -> Stage {
        self.stage.as_str().into()
    }
}
