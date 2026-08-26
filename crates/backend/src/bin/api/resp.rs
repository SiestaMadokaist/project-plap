use domain::errors::DomainError;
use dto::response::{Placeholder, ToResp, DTO};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
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
