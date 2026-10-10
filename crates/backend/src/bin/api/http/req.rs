use axum::{
    body::Bytes,
    http::{HeaderMap, Method},
};
use domain::errors::DomainError;
use pkg::{auth::claims::JWT, id::TraceId};

/// The bits of an incoming request the route handlers care about, pulled out of
/// axum's extractors so `routes/*` doesn't depend on axum types.
pub struct HttpEvent {
    _method: Method,
    path: String,
    headers: HeaderMap,
    body: Bytes,
}

impl HttpEvent {
    pub fn new(_method: Method, path: String, headers: HeaderMap, body: Bytes) -> Self {
        Self {
            _method,
            path,
            headers,
            body,
        }
    }

    /// Header value as `&str`; `HeaderMap` lookups are already case-insensitive.
    fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).and_then(|v| v.to_str().ok())
    }

    pub fn trace_id(&self) -> Option<TraceId> {
        let id = self
            .header("x-trace-id")
            .map(TraceId::new)
            .or_else(|| Some(TraceId::default()));
        id
    }

    /// Bearer token from the `Authorization` header (optional `Bearer ` prefix).
    /// Only the authorized-route path calls this.
    pub fn authorization(&self) -> Result<JWT, DomainError> {
        let raw = self
            .header("authorization")
            .ok_or_else(|| DomainError::NotAllowed("missing authorization header".into()))?;
        let token = raw.strip_prefix("Bearer ").unwrap_or(raw).trim();
        Ok(JWT(token.into()))
    }

    /// Parsed request body. An absent or empty body is treated as `{}`.
    pub fn body(&self) -> Result<serde_json::Value, DomainError> {
        if self.body.is_empty() {
            return Ok(serde_json::json!({}));
        }
        serde_json::from_slice(&self.body).map_err(|e| DomainError::Serialize(e.to_string()))
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}
