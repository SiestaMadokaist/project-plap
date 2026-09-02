use std::collections::HashMap;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use domain::errors::DomainError;
use lambda_runtime::LambdaEvent;
use pkg::auth::claims::JWT;
use serde::Deserialize;

/// API Gateway HTTP API proxy event, payload format **2.0** only. A v1/REST event
/// (which carries `path` instead of `rawPath`) fails to deserialize here by design.
#[derive(Deserialize, Debug)]
pub struct ApiEvent {
    #[serde(rename = "rawPath")]
    raw_path: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    body: Option<String>,
    #[serde(rename = "isBase64Encoded", default)]
    is_base64_encoded: bool,
    /// `requestContext.http` carries the method. Defaulted so a hand-rolled local
    /// fixture without it still deserializes.
    #[serde(rename = "requestContext", default)]
    request_context: RequestContext,
}

#[derive(Deserialize, Debug, Default)]
struct RequestContext {
    #[serde(default)]
    http: HttpDescription,
}

#[derive(Deserialize, Debug, Default)]
struct HttpDescription {
    #[serde(default)]
    method: String,
}

impl ApiEvent {
    /// Case-insensitive header lookup.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    /// HTTP verb (`GET`, `POST`, `OPTIONS`, …). Empty string if the event omitted it.
    pub fn method(&self) -> &str {
        &self.request_context.http.method
    }
}

#[derive(Debug)]
pub struct HttpEvent(pub LambdaEvent<ApiEvent>);

impl HttpEvent {
    /// Bearer token from the `Authorization` header (case-insensitive key, optional
    /// `Bearer ` prefix). Only the authorized-route path calls this.
    pub fn authorization(&self) -> Result<JWT, DomainError> {
        let raw = self
            .0
            .payload
            .header("authorization")
            .ok_or_else(|| DomainError::NotAllowed("missing authorization header".into()))?;
        let token = raw.strip_prefix("Bearer ").unwrap_or(raw).trim();
        Ok(JWT(token.into()))
    }

    /// Parsed request body. An absent or empty body is treated as `{}`. Honours
    /// `isBase64Encoded`, which API Gateway sets for non-text content types.
    pub fn body(&self) -> Result<serde_json::Value, DomainError> {
        let payload = &self.0.payload;
        let raw = match payload.body.as_deref() {
            None | Some("") => return Ok(serde_json::json!({})),
            Some(b) => b,
        };
        let bytes = if payload.is_base64_encoded {
            STANDARD
                .decode(raw)
                .map_err(|e| DomainError::Serialize(e.to_string()))?
        } else {
            raw.as_bytes().to_vec()
        };
        serde_json::from_slice(&bytes).map_err(|e| DomainError::Serialize(e.to_string()))
    }

    pub fn path(&self) -> &str {
        &self.0.payload.raw_path
    }

    pub fn method(&self) -> &str {
        self.0.payload.method()
    }
}
