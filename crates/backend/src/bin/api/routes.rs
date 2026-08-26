use std::collections::HashMap;

use lambda_runtime::LambdaEvent;
use domain::errors::DomainError;
use pkg::displayable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    // QUERY,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum RouteId {
    #[serde(rename = "/models/list")]
    ListModels,
    #[serde(rename = "/agent/command/fetch")]
    AgentCommandFetchModel,
    #[serde(rename = "/test")]
    TestEndpoint,
}
displayable!(RouteId);

#[derive(Serialize)]
pub struct ApiResponse {
    #[serde(rename = "statusCode")]
    status_code: u16,
    headers: HashMap<String, String>,
    body: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse<'a> {
    err: &'a DomainError,
    code: u16,
}
pub fn error_code(err: &DomainError) -> u16 {
    match err {
        DomainError::ApiError(_) => 503,
        DomainError::NotAllowed(_) => 403,
        _ => 500,
    }
}

pub fn err_response(err: &DomainError) -> ApiResponse {
    let code = error_code(err);
    let resp = ErrorResponse { err, code };
    tracing::error!("error: {}", err.to_string());
    let stringified = serde_json::to_string(&resp);
    match stringified {
        Ok(s) => json_response(code, s),
        Err(e) => {
            tracing::error!("error serialize: {}", e);
            json_response(500, r#"{"error": "internal error"}"#)
        }
    }
}

pub fn json_response(status_code: u16, body: impl Into<String>) -> ApiResponse {
    let mut headers = HashMap::new();
    headers.insert("content-type".into(), "application/json".into());
    ApiResponse {
        status_code,
        headers,
        body: body.into(),
    }
}

#[derive(Deserialize)]
pub struct ApiEvent {
    path: String,
    #[serde(rename = "httpMethod")]
    #[allow(dead_code)]
    http_method: HttpMethod,
    #[allow(dead_code)]
    body: Option<String>,
}

// type HttpEvent = LambdaEvent<ApiEvent>;
pub struct HttpEvent(pub LambdaEvent<ApiEvent>);

impl HttpEvent {
    pub fn body(&self) -> Result<serde_json::Value, serde_json::Error> {
        let body = &self.0.payload.body;
        match body {
            None => Ok(serde_json::from_str("{}").expect("{} is always a valid json}")),
            Some(x) => {
                let result = serde_json::from_str::<serde_json::Value>(x)?;
                Ok(result)
            }
        }
    }

    pub fn path(&self) -> &str {
        &self.0.payload.path
    }
}
