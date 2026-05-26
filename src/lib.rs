pub mod application;
pub mod bootstrap;
pub mod domain;
pub mod infras;
pub mod pkg;

pub fn init_env() {
    if let Ok(path) = std::env::var("ENV_PATH") {
        let _ = dotenvy::from_filename(path);
    }
}

pub fn init_tracing() {
    let is_local = std::env::var("AWS_LAMBDA_RUNTIME_API").is_err();
    let filter = tracing_subscriber::EnvFilter::from_default_env();

    if is_local {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .pretty()
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .json()
            .without_time()
            .init();
    }
}
