use std::collections::HashMap;

use lambda_runtime::LambdaEvent;
use pkg::{auth::claims::JWT, displayable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    // QUERY,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum AuthorizedRoute {
    #[serde(rename = "/models/list")]
    ListModels,
    #[serde(rename = "/agents/command/fetch")]
    AgentCommandFetchModel,
    #[serde(rename = "/test")]
    TestEndpoint,
}
displayable!(AuthorizedRoute);

#[derive(Debug, Serialize, Deserialize)]
pub enum PublicRoute {
    #[serde(rename = "/users/login")]
    UserLogin,
}

#[derive(Deserialize)]
pub struct ApiEvent<Auth> {
    path: String,
    #[serde(rename = "httpMethod")]
    #[allow(dead_code)]
    http_method: HttpMethod,
    #[allow(dead_code)]
    body: Option<String>,
    auth: Option<Auth>,
}

pub struct HttpEvent<A>(pub LambdaEvent<ApiEvent<A>>);

impl HttpEvent<JWT> {
    pub fn authorization(&self) -> JWT {
        // self.0.payload.
        todo!();
    }

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
