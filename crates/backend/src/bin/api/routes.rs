use lambda_runtime::LambdaEvent;
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

#[derive(Deserialize)]
pub struct ApiEvent {
    path: String,
    #[serde(rename = "httpMethod")]
    #[allow(dead_code)]
    http_method: HttpMethod,
    #[allow(dead_code)]
    body: Option<String>,
}

pub struct HttpEvent(pub LambdaEvent<ApiEvent>);

impl HttpEvent {
    pub fn body(&self) -> Result<serde_json::Value, serde_json::Error> {
        let body = &self.0.payload.body;
        match body {
            None => Ok(serde_json::from_str("{}").expect("{} is always a valid json")),
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
