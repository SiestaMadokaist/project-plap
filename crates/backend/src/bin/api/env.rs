use domain::{commands::compute::ComputeRegion, storage::StoragePrefix};
use pkg::{
    enums::stage::Stage,
    types::{
        crypto::EthPriv,
        strings::URL,
        time::{Second, Timestamp},
    },
    utils::{var_or, var_second},
};
use std::env;

#[derive(Debug)]
pub struct ApiEnv {
    stage: String,
    sanity_run: String,

    /// `host:port` the HTTP server binds to.
    pub bind_addr: String,
    /// Allowed browser origin for CORS (the frontend). Unset = any origin (local dev).
    pub cors_origin: Option<String>,

    pub sentry_dsn: Option<String>,
    pub sentry_env: String,

    pub discord_username: String,
    pub discord_webhook_url: URL,

    pub auth_secret: String,
    pub auth_privkey: EthPriv,
    pub auth_challenge_ttl: Second,
    pub auth_session_ttl: Second,
    pub auth_clock_skew: Second,
    pub auth_min_iat: Timestamp,

    pub output_region: String,
    pub output_bucket: String,
    pub output_rootdir: StoragePrefix,

    pub model_region: String,
    pub model_bucket: String,
    pub model_rootdir: StoragePrefix,

    pub template_region: ComputeRegion,
    pub template_bucket: String,
    pub template_rootdir: StoragePrefix,
}

impl ApiEnv {
    pub fn from_env() -> Self {
        Self {
            sanity_run: var_or("SANITY_RUN", "false"),
            stage: env::var("STAGE").expect("stage must be set"),

            bind_addr: var_or("BIND_ADDR", "0.0.0.0:3000"),
            cors_origin: env::var("CORS_ORIGIN").ok().filter(|s| !s.is_empty()),

            sentry_dsn: env::var("SENTRY_DSN").ok().filter(|s| !s.is_empty()),
            sentry_env: var_or("SENTRY_ENV", "unknown"),
            discord_username: var_or("DISCORD_USERNAME", "lambda-api"),
            discord_webhook_url: env::var("DISCORD_WEBHOOK_URL")
                .map(URL)
                .expect("DISCORD_WEBHOOK_URL must be set"),

            auth_secret: env::var("AUTH_SECRET").expect("AUTH_SECRET must be set"),
            auth_privkey: env::var("AUTH_PRIVKEY")
                .map(|x| x.as_str().try_into().expect("AUTH PRIVKEY must be valid"))
                .expect("AUTH_PRIVKEY must be set"),
            auth_challenge_ttl: var_second("AUTH_CHALLENGE_TTL"),
            auth_session_ttl: var_second("AUTH_SESSION_TTL"),
            auth_clock_skew: var_second("AUTH_CLOCK_SKEW"),
            auth_min_iat: Timestamp(
                var_or("AUTH_MIN_IAT", "0")
                    .parse()
                    .expect("AUTH_MIN_IAT must be an integer"),
            ),
            template_region: env::var("TEMPLATE_REGION")
                .and_then(ComputeRegion::from_env)
                .expect("TEMPLATE_REGION must be set"),
            template_bucket: env::var("TEMPLATE_BUCKET").expect("TEMPLATE_BUCKET must be set"),
            template_rootdir: env::var("TEMPLATE_ROOTDIR")
                .map(StoragePrefix)
                .expect("TEMPLATE_ROOTDIR must be set"),

            output_region: var_or("OUTPUT_REGION", "ap-southeast-1"),
            output_bucket: env::var("OUTPUT_BUCKET").expect("OUTPUT_BUCKET must be set"),
            output_rootdir: env::var("OUTPUT_ROOTDIR")
                .map(StoragePrefix)
                .expect("OUTPUT_ROOTDIR must be set"),

            model_region: env::var("MODEL_REGION").expect("MODEL_REGION must be set"),
            model_bucket: env::var("MODEL_BUCKET").expect("MODEL_BUCKET must be set"),
            model_rootdir: env::var("MODEL_ROOTDIR")
                .map(StoragePrefix)
                .expect("MODEL_ROOTDIR must be set"),
        }
    }

    pub fn sanity_run(&self) -> bool {
        self.sanity_run == "true"
    }

    pub fn stage(&self) -> Stage {
        self.stage.as_str().into()
    }
}
