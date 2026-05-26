pub mod application;
pub mod bootstrap;
pub mod domain;
pub mod infras;
pub mod pkg;

pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .without_time()
        .init();
}
