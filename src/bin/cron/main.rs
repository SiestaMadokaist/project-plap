pub mod resources;
use std::any::{self, Any};

use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use rust_api::{
    application::ports::{clients::cc::AllClient, repository::rc::AllRepos},
    domain::errors::DomainError,
};
use serde::Deserialize;

use crate::resources::translations::controller::TranslationController;

#[derive(Deserialize)]
struct CronEvent {
    #[serde(rename = "pathParameters")]
    path_parameters: PathParameters,
}

#[derive(Deserialize)]
struct PathParameters {
    proxy: String,
}

async fn handler(event: LambdaEvent<CronEvent>) -> Result<(), Error> {
    let proxy = &event.payload.path_parameters.proxy;
    // let c = controller<Any; Any>();
    // let translation = TranslationController::new(repo, client);
    tracing::info!(proxy, "cron event");
    match proxy.as_str() {
        "cron/rezero" => {}
        _ => tracing::warn!(proxy, "unknown cron route"),
    }

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    rust_api::init_tracing();
    run(service_fn(handler)).await
}
