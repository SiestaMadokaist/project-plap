use domain::storage::StoragePrefix;
use pkg::{
    enums::stage::Stage,
    types::{
        strings::{Hex, URL},
        time::{Second, Timestamp},
    },
    utils::{var_or, var_second},
};
use std::env;

pub struct ApiEnv {
    stage: String,
    sanity_run: String,

    pub discord_username: String,
    pub discord_webhook_url: URL,

    pub auth_secret: String,
    pub auth_privkey: Hex,
    pub auth_challenge_ttl: Second,
    pub auth_session_ttl: Second,
    pub auth_clock_skew: Second,
    pub auth_min_iat: Timestamp,

    pub output_region: String,
    pub output_bucket: String,
    pub output_prefix: StoragePrefix,

    pub model_region: String,
    pub model_bucket: String,
    pub model_prefix: StoragePrefix,
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

            auth_secret: env::var("AUTH_SECRET").expect("AUTH_SECRET must be set"),
            auth_privkey: env::var("AUTH_PRIVKEY")
                .map(Hex)
                .expect("AUTH_PRIVKEY must be set"),
            auth_challenge_ttl: var_second("AUTH_CHALLENGE_TTL"),
            auth_session_ttl: var_second("AUTH_SESSION_TTL"),
            auth_clock_skew: var_second("AUTH_CLOCK_SKEW"),
            auth_min_iat: Timestamp(
                var_or("AUTH_MIN_IAT", "0")
                    .parse()
                    .expect("AUTH_MIN_IAT must be an integer"),
            ),

            output_region: var_or("OUTPUT_REGION", "ap-southeast-1"),
            output_bucket: env::var("OUTPUT_BUCKET").expect("OUTPUT_BUCKET must be set"),
            output_prefix: env::var("OUTPUT_PREFIX")
                .map(StoragePrefix)
                .expect("OUTPUT PREFIX must be set"),

            model_region: env::var("MODEL_REGION").expect("MODEL_REGION must be set"),
            model_bucket: env::var("MODEL_BUCKET").expect("MODEL_BUCKET must be set"),
            model_prefix: env::var("MODEL_PREFIX")
                .map(StoragePrefix)
                .expect("MODEL PREFIX must be set"),
        }
    }

    pub fn sanity_run(&self) -> bool {
        self.sanity_run == "true"
    }

    pub fn stage(&self) -> Stage {
        self.stage.as_str().into()
    }
}
