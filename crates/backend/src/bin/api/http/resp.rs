use axum::{
    http::{HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
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

impl IntoResponse for ServerResponse {
    fn into_response(self) -> Response {
        let status =
            StatusCode::from_u16(self.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let mut resp = (status, self.body).into_response();
        for (k, v) in self.headers {
            if let (Ok(k), Ok(v)) = (HeaderName::try_from(k), HeaderValue::try_from(v)) {
                resp.headers_mut().insert(k, v);
            }
        }
        resp
    }
}
