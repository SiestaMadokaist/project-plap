pub mod application;
pub mod constant;
pub mod infras;
pub mod trigger;

pub fn init_env() {
    if let Ok(path) = std::env::var("ENV_PATH") {
        let _ = dotenvy::from_filename(path);
    }
}

/// Starts the Sentry client. `dsn` / `environment` come from each binary's own
/// env struct (see the `env.rs` files). Returns the guard, which flushes buffered
/// events when dropped — `main` must keep it bound for the whole process (a bare
/// `_` would drop it immediately and lose every event). A no-op returning `None`
/// when `dsn` is absent/empty, so local runs and tests need no Sentry config.
pub fn init_sentry(dsn: Option<&str>, environment: &str) -> Option<sentry::ClientInitGuard> {
    let dsn = dsn.filter(|d| !d.is_empty())?;

    // ClientOptions is #[non_exhaustive], so it can't be built with a struct literal.
    let mut options = sentry::ClientOptions::new();
    options.release = sentry::release_name!();
    options.environment = Some(environment.to_owned().into());

    Some(sentry::init((dsn.to_owned(), options)))
}

pub fn init_tracing() {
    use tracing_subscriber::prelude::*;

    let is_local = std::env::var("AWS_LAMBDA_RUNTIME_API").is_err();
    let filter = tracing_subscriber::EnvFilter::from_default_env();

    let fmt_layer = tracing_subscriber::fmt::layer();
    let fmt_layer = if is_local {
        fmt_layer.pretty().boxed()
    } else {
        fmt_layer.json().without_time().boxed()
    };

    // sentry_tracing::layer() forwards `error!` events to Sentry as events and
    // lower levels as breadcrumbs; it's inert when no Sentry client is running.
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .with(sentry_tracing::layer())
        .init();
}
