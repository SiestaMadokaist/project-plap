pub mod routes;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use rust_api::pkg::types::time::Timestamp;
use serde::Deserialize;

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
    tracing::info!(proxy, "cron event");
    match proxy.as_str() {
        "cron/ocr-service" => {}
        "cron/mldn" => {}
        "cron/autosync" => {}
        _ => tracing::warn!(proxy, "unknown cron route"),
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    rust_api::init_tracing();
    run(service_fn(handler)).await
}
