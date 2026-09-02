use std::rc::Rc;

use backend::application::{
    ports::clients::{authorizer::Authorizer, container::HasAuthValidator},
    routes::authorized::AuthorizedRoute,
};
use domain::{ctx, errors::DomainError};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use matchit::Router;

mod bootstrap;
mod env;
mod http;
mod routes;

use bootstrap::{client::ApiClients, repo::ApiRepos};
use env::ApiEnv;
use pkg::auth::claims::JWT;

use crate::{
    http::{
        req::{ApiEvent, HttpEvent},
        resp::{no, no_content, ServerResponse},
    },
    routes::{
        authorized::{authorized_routes, handle_authorized},
        public::{handle_public, public_routes},
    },
};

async fn init_ctx(clients: &ApiClients, token: JWT) -> Result<ctx::Context, DomainError> {
    let claims = clients.authorizer().validate(token).await?;
    let ctx = ctx::Context::new(claims, None);
    Ok(ctx)
}

/// Resolves the caller's context from the `Authorization` header before dispatching
/// to an authorized route. Keeping `init_ctx` here means the route handler only ever
/// sees a ready `&Context`.
async fn route_authorized(
    repos: Rc<ApiRepos>,
    clients: Rc<ApiClients>,
    router: Rc<Router<AuthorizedRoute>>,
    event: HttpEvent,
) -> Result<ServerResponse, DomainError> {
    let ctx = init_ctx(clients.as_ref(), event.authorization()?).await?;
    handle_authorized(repos, clients, router, &ctx, event).await
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    backend::init_env();
    backend::init_tracing();

    let env = ApiEnv::from_env();
    if env.sanity_run() {
        return Ok(());
    }
    let config = aws_config::from_env().load().await;
    let repo = ApiRepos::rc(&env, &config);
    let client = ApiClients::rc(&env, &config);
    let authorized_router = Rc::new(authorized_routes());
    let public_router = Rc::new(public_routes());

    run(service_fn(move |event: LambdaEvent<ApiEvent>| {
        let r = repo.clone();
        let c = client.clone();
        let public_rt = public_router.clone();
        let authorized_rt = authorized_router.clone();
        let http_event = HttpEvent(event);
        async move {
            // CORS preflight: the $default route hands OPTIONS to us; answer 204 and
            // let API Gateway's cors_configuration attach the Access-Control-* headers.
            if http_event.method().eq_ignore_ascii_case("OPTIONS") {
                return Ok::<ServerResponse, Error>(no_content());
            }

            let path = http_event.path().to_string();
            // check public first because authorized might throw error if unauthorized
            let handled: Result<ServerResponse, DomainError> = if public_rt.at(&path).is_ok() {
                handle_public(r, c, public_rt, http_event).await
            } else {
                route_authorized(r, c, authorized_rt, http_event).await
            };
            let converted: ServerResponse = match handled {
                Ok(x) => x,
                Err(e) => {
                    tracing::error!("unhandled exception: {}", e);
                    no(e)
                }
            };
            Ok::<ServerResponse, Error>(converted)
        }
    }))
    .await
}
