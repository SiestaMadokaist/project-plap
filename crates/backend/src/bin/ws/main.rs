use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde_json::{json, Value};

mod bootstrap;

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
    backend::init_tracing();
    run(service_fn(handler)).await
}
