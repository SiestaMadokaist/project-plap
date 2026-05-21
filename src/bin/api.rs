use lambda_http::{run, service_fn, Body, Error, Request, Response};

async fn handler(event: Request) -> Result<Response<Body>, Error> {
    let path = event.uri().path().to_owned();
    let method = event.method().to_string();
    tracing::info!(path, method, "request");

    let resp = Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(Body::from(r#"{"status":"ok"}"#))?;

    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    rust_api::init_tracing();
    run(service_fn(handler)).await
}
