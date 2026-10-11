use axum::{middleware::from_fn_with_state, Router};
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

type Error = Box<dyn std::error::Error + Send + Sync>;

mod bootstrap;
mod env;
mod http;
mod routes;

use bootstrap::{client::ApiClients, repo::ApiRepos};
use env::ApiEnv;
use http::{
    app::AppState,
    middleware::{authorize, guest},
};

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
    };

    let public = routes::public::router().route_layer(from_fn_with_state(state.clone(), guest));
    let authorized =
        routes::authorized::router().route_layer(from_fn_with_state(state.clone(), authorize));
    let app = Router::new()
        .merge(public)
        .merge(authorized)
        .layer(cors(env)?)
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(&env.bind_addr).await?;
    tracing::info!("listening on {}", env.bind_addr);
    axum::serve(listener, app).await?;
    Ok(())
}
