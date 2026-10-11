use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{FromRequest, FromRequestParts, Request},
    http::request::Parts,
    response::{IntoResponse, Response},
};
use domain::{ctx, errors::DomainError};
use dto::response::{ToResp, DTO};

use super::resp::{no, yes, ServerResponse};
use crate::bootstrap::{client::ApiClients, repo::ApiRepos};

/// Shared by every handler; cloning is cheap since it is all `Arc`.
#[derive(Clone)]
pub struct AppState {
    pub repos: Arc<ApiRepos>,
    pub clients: Arc<ApiClients<'static, ()>>,
}

/// What every handler returns.
pub type Reply = Result<ServerResponse, ApiError>;

/// `DomainError` lives in another crate, so it can't implement axum's `IntoResponse`
/// here (orphan rule). This local wrapper can, and `From` lets handlers use `?`.
pub struct ApiError(pub DomainError);

impl From<DomainError> for ApiError {
    fn from(e: DomainError) -> Self {
        ApiError(e)
    }
}

/// `payload.try_into()?` fails with a serde error; go through `DomainError`'s own conversion.
impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        ApiError(DomainError::from(e))
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        tracing::error!("unhandled exception: {}", self.0);
        no(self.0).into_response()
    }
}

/// Turns a usecase result into the JSON envelope (`success`/`failure` + http code).
pub fn reply<D: DTO>(result: Result<D, DomainError>) -> Reply {
    result.to_result().and_then(yes).map_err(ApiError)
}

/// The `Context` the auth/guest middleware stored for this request.
pub struct Ctx(pub ctx::Context);

impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = ApiError;
    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, ApiError> {
        parts
            .extensions
            .get::<ctx::Context>()
            .cloned()
            .map(Ctx)
            .ok_or(ApiError(DomainError::Unhandled))
    }
}

/// Request body as JSON. An absent or empty body is treated as `{}`, so the
/// usecases' `payload.try_into()?` sees the same thing it did behind API Gateway.
pub struct Payload(pub serde_json::Value);

impl<S: Send + Sync> FromRequest<S> for Payload {
    type Rejection = ApiError;
    async fn from_request(req: Request, state: &S) -> Result<Self, ApiError> {
        let bytes = Bytes::from_request(req, state)
            .await
            .map_err(|e| ApiError(DomainError::Serialize(e.to_string())))?;
        if bytes.is_empty() {
            return Ok(Payload(serde_json::json!({})));
        }
        serde_json::from_slice(&bytes)
            .map(Payload)
            .map_err(|e| ApiError(DomainError::Serialize(e.to_string())))
    }
}
