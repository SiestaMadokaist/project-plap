use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde_json::{json, Value};

mod bootstrap;
mod env;

use env::WsEnv;

async fn handler(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let route_key = event.payload["requestContext"]["routeKey"]
        .as_str()
        .unwrap_or("$default");
    let connection_id = event.payload["requestContext"]["connectionId"]
        .as_str()
        .unwrap_or("");

    tracing::info!(route_key, connection_id, "websocket event");

    match route_key {
        "$connect" => {}
        "$disconnect" => {}
        _ => {}
    }

    Ok(json!({"statusCode": 200}))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    backend::init_env();
    let env = WsEnv::from_env();
    let _sentry = backend::init_sentry(env.sentry_dsn.as_deref(), &env.sentry_env);
    backend::init_tracing();
    run(service_fn(handler)).await
}
