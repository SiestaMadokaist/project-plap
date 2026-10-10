use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, Method, Uri},
    Router as AxumRouter,
};
use backend::application::{
    ports::clients::{authorizer::Authorizer, container::HasAuthValidator},
    routes::{authorized::AuthorizedRoute, public::PublicRoute},
};
use domain::{ctx, errors::DomainError};
use matchit::Router;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

type Error = Box<dyn std::error::Error + Send + Sync>;

mod bootstrap;
mod env;
mod http;
mod routes;

use bootstrap::{client::ApiClients, repo::ApiRepos};
use env::ApiEnv;
use pkg::{auth::claims::JWT, id::TraceId};
use tracing::Instrument;

use crate::{
    http::{
        req::HttpEvent,
        resp::{no, ServerResponse},
    },
    routes::{
        authorized::{authorized_routes, handle_authorized},
        public::{handle_public, public_routes},
    },
};

async fn init_ctx(
    clients: &ApiClients<'_, ()>,
    token: JWT,
    trace_id: Option<TraceId>,
) -> Result<ctx::Context, DomainError> {
    let claims = clients.authorizer().validate(token).await?;
    let ctx = ctx::Context::new(claims, trace_id);
    Ok(ctx)
}

/// Resolves the caller's context from the `Authorization` header before dispatching
/// to an authorized route. Keeping `init_ctx` here means the route handler only ever
/// sees a ready `&Context`.
async fn route_authorized(
    repos: Arc<ApiRepos>,
    clients: &ApiClients<'_, ()>,
    router: Arc<Router<AuthorizedRoute>>,
    event: HttpEvent,
) -> Result<ServerResponse, DomainError> {
    let ctx = init_ctx(clients, event.authorization()?, event.trace_id()).await?;
    let api_clients = clients.start(&ctx);
    let trace_id = ctx.trace_id();
    let span = tracing::info_span!("request", trace_id = %trace_id);
    handle_authorized(repos, &api_clients, router, &ctx, event)
        .instrument(span)
        .await
}

/// Resolves the caller's context from the `Authorization` header before dispatching
/// to an authorized route. Keeping `init_ctx` here means the route handler only ever
/// sees a ready `&Context`.
async fn route_public(
    repos: Arc<ApiRepos>,
    clients: &ApiClients<'_, ()>,
    router: Arc<Router<PublicRoute>>,
    event: HttpEvent,
) -> Result<ServerResponse, DomainError> {
    let ctx = ctx::Context::guest(event.trace_id());
    let api_clients = clients.start(&ctx);
    let trace_id = ctx.trace_id();
    let span = tracing::info_span!("request", trace_id = %trace_id);
    handle_public(repos, &api_clients, router, event)
        .instrument(span)
        .await
}

/// Everything a request handler needs, cloned (cheaply, it is all `Arc`) per request.
#[derive(Clone)]
struct AppState {
    repos: Arc<ApiRepos>,
    clients: Arc<ApiClients<'static, ()>>,
    public_router: Arc<Router<PublicRoute>>,
    authorized_router: Arc<Router<AuthorizedRoute>>,
}

/// Single catch-all handler: axum does the HTTP, the `matchit` routers keep doing
/// the path -> usecase dispatch exactly as before.
#[axum::debug_handler]
async fn dispatch(
    State(app): State<AppState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> ServerResponse {
    let path = uri.path().to_string();
    let http_event = HttpEvent::new(method, path.clone(), headers, body);
    // check public first because authorized might throw error if unauthorized
    let handled: Result<ServerResponse, DomainError> = if app.public_router.at(&path).is_ok() {
        route_public(app.repos, &app.clients, app.public_router, http_event).await
    } else {
        route_authorized(app.repos, &app.clients, app.authorized_router, http_event).await
    };
    match handled {
        Ok(x) => x,
        Err(e) => {
            tracing::error!("unhandled exception: {}", e);
            no(e)
        }
    }
}

fn cors(env: &ApiEnv) -> Result<CorsLayer, Error> {
    let origin = match &env.cors_origin {
        Some(o) => AllowOrigin::exact(o.parse()?),
        None => AllowOrigin::any(),
    };
    Ok(CorsLayer::new()
        .allow_origin(origin)
        .allow_methods(Any)
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
        ])
        .max_age(std::time::Duration::from_secs(86400)))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    backend::init_env();
    // leaked on purpose: they live for the whole process, and axum's state must be 'static
    let env: &'static ApiEnv = Box::leak(Box::new(ApiEnv::from_env()));
    let _sentry = backend::init_sentry(env.sentry_dsn.as_deref(), &env.sentry_env);
    backend::init_tracing();

    if env.sanity_run() {
        return Ok(());
    }
    let config: &'static _ = Box::leak(Box::new(aws_config::from_env().load().await));
    let state = AppState {
        repos: ApiRepos::arc(env, config),
        clients: ApiClients::arc(env, config),
        public_router: Arc::new(public_routes()),
        authorized_router: Arc::new(authorized_routes()),
    };

    let app = AxumRouter::new()
        .fallback(dispatch)
        .layer(cors(env)?)
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(&env.bind_addr).await?;
    tracing::info!("listening on {}", env.bind_addr);
    axum::serve(listener, app).await?;
    Ok(())
}
