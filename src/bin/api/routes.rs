use std::collections::HashMap;

use lambda_runtime::LambdaEvent;
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
}

#[derive(Serialize)]
pub struct ApiResponse {
    #[serde(rename = "statusCode")]
    status_code: u16,
    headers: HashMap<String, String>,
    body: String,
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
    body: Option<serde_json::Value>,
}

// type HttpEvent = LambdaEvent<ApiEvent>;
pub struct HttpEvent(pub LambdaEvent<ApiEvent>);

impl HttpEvent {
    pub fn body(&self) -> Option<serde_json::Value> {
        self.0.payload.body.clone()
    }
}
