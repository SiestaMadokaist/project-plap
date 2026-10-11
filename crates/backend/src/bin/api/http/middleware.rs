use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::{IntoResponse, Response},
};
use backend::application::ports::clients::{authorizer::Authorizer, container::HasAuthValidator};
use domain::{ctx, errors::DomainError};
use pkg::{auth::claims::JWT, id::TraceId};
use tracing::Instrument;

use super::app::{ApiError, AppState};

fn trace_id(headers: &HeaderMap) -> Option<TraceId> {
    headers
        .get("x-trace-id")
        .and_then(|v| v.to_str().ok())
        .map(TraceId::new)
}

/// Bearer token from the `Authorization` header (optional `Bearer ` prefix).
fn token(headers: &HeaderMap) -> Result<JWT, DomainError> {
    let raw = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| DomainError::NotAllowed("missing authorization header".into()))?;
    Ok(JWT(raw
        .strip_prefix("Bearer ")
        .unwrap_or(raw)
        .trim()
        .into()))
}

/// Validates the caller, then hands the handler a ready `Context` (see `Ctx`).
pub async fn authorize(State(app): State<AppState>, req: Request, next: Next) -> Response {
    // read what we need from `req` up front: holding `&req` across the `.await` below
    // would make this future `!Send` (a request body is not `Sync`).
    let token = token(req.headers());
    let trace_id = trace_id(req.headers());
    let built = async {
        let claims = app.clients.authorizer().validate(token?).await?;
        Ok::<_, DomainError>(ctx::Context::new(claims, trace_id))
    }
    .await;
    with_ctx(built, req, next).await
}

/// Public routes get a guest `Context`.
pub async fn guest(req: Request, next: Next) -> Response {
    let built = Ok(ctx::Context::guest(trace_id(req.headers())));
    with_ctx(built, req, next).await
}

async fn with_ctx(
    built: Result<ctx::Context, DomainError>,
    mut req: Request,
    next: Next,
) -> Response {
    let ctx = match built {
        Ok(c) => c,
        Err(e) => return ApiError(e).into_response(),
    };
    let span = tracing::info_span!("request", trace_id = %ctx.trace_id());
    req.extensions_mut().insert(ctx);
    next.run(req).instrument(span).await
}
