use std::env;

use pkg::utils::var_or;

/// The ws lambda needs almost nothing from the environment yet — just the Sentry
/// wiring, kept here so every binary reads its config through one `env.rs`.
pub struct WsEnv {
    pub sentry_dsn: Option<String>,
    pub sentry_env: String,
}

impl WsEnv {
    pub fn from_env() -> Self {
        Self {
            sentry_dsn: env::var("SENTRY_DSN").ok().filter(|s| !s.is_empty()),
            sentry_env: var_or("SENTRY_ENV", "unknown"),
        }
    }
}
