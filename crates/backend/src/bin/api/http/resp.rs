use domain::errors::DomainError;
use dto::response::{Placeholder, ToResp};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerResponse {
    #[serde(rename = "statusCode")]
    status_code: u16,
    headers: HashMap<String, String>,
    body: String,
}

pub fn yes(
    data: dto::response::Response<serde_json::Value>,
) -> Result<ServerResponse, DomainError> {
    let mut headers = HashMap::new();
    headers.insert("content-type".into(), "application/json".into());
    let response = ServerResponse {
        status_code: data.httpcode(),
        headers,
        body: data.to_body(),
    };
    Ok(response)
}

/// Empty `204`. Used to answer the CORS preflight — the `$default` route hands
/// `OPTIONS` to this Lambda, and API Gateway's `cors_configuration` then decorates
/// the response with the `Access-Control-*` headers.
pub fn no_content() -> ServerResponse {
    ServerResponse {
        status_code: 204,
        headers: HashMap::new(),
        body: String::new(),
    }
}

pub fn no(error: DomainError) -> ServerResponse {
    let mut headers = HashMap::new();
    headers.insert("content-type".into(), "application/json".into());
    let err: Result<Placeholder, DomainError> = Err(error);
    let resp = err.to_resp();
    let response = ServerResponse {
        status_code: resp.httpcode(),
        headers,
        body: resp.to_body(),
    };
    response
}
