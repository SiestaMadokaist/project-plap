use std::rc::Rc;

use backend::application::{
    ports::usecase::UsecaseAPI,
    usecases::hq::{commands::cp_model::CPModel, models::list::GetList},
};
use domain::errors::DomainError;
use dto::response::{Placeholder, ToResp};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use matchit::Router;

mod bootstrap;
mod env;
mod req;
mod resp;

use bootstrap::{client::ApiClients, repo::ApiRepos};
use env::ApiEnv;
use pkg::auth::claims::JWT;

use crate::{
    req::{ApiEvent, AuthorizedRoute, HttpEvent, PublicRoute},
    resp::{no, yes, ServerResponse},
};

fn routes() -> Router<AuthorizedRoute> {
    let mut router = Router::new();
    let expectation = "must be valid route";
    router
        .insert(
            AuthorizedRoute::ListModels.to_string(),
            AuthorizedRoute::ListModels,
        )
        .expect(expectation);
    router
        .insert(
            AuthorizedRoute::AgentCommandFetchModel,
            AuthorizedRoute::AgentCommandFetchModel,
        )
        .expect(expectation);
    router
}

async fn handle_public(
    repos: Rc<ApiRepos>,
    clients: Rc<ApiClients>,
    router: Rc<Router<PublicRoute>>,
    event: HttpEvent<()>,
) -> Result<ServerResponse, DomainError> {
    Err(DomainError::EmptyResponse)
}

async fn handle_authorized(
    repos: Rc<ApiRepos>,
    clients: Rc<ApiClients>,
    router: Rc<Router<AuthorizedRoute>>,
    event: HttpEvent<JWT>,
) -> Result<ServerResponse, DomainError> {
    let path = event.path();
    let auth = event.authorization();
    let route = router.at(path);
    let resp: Result<dto::response::Response<serde_json::Value>, DomainError> = match route {
        Err(x) => Err(DomainError::Prerequisite(x.to_string())),
        Ok(matched) => match matched.value {
            AuthorizedRoute::ListModels => GetList::new(clients.clone(), event.body()?.try_into()?)
                .exec()
                .await
                .to_result(),
            AuthorizedRoute::AgentCommandFetchModel => {
                CPModel::new(repos.clone(), event.body()?.try_into()?)
                    .exec()
                    .await
                    .to_result()
            }
            _ => Err::<Placeholder, DomainError>(DomainError::NotFound).to_result(),
        },
    };
    resp.and_then(yes)
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
    let dynamo = aws_sdk_dynamodb::Client::new(&config);

    let repo = ApiRepos::rc(&dynamo, env.stage());
    let client = ApiClients::rc(&env, &config);
    let router = Rc::new(routes());

    // todo: ApiEvent<JWT??? here
    run(service_fn(move |event: LambdaEvent<ApiEvent<JWT>>| {
        let r = repo.clone();
        let c = client.clone();
        let rt = router.clone();
        let http_event = HttpEvent(event);
        async move {
            let handled: Result<ServerResponse, DomainError> =
                handle_authorized(r, c, rt, http_event).await;
            let converted: ServerResponse = match handled {
                Ok(x) => x,
                Err(e) => {
                    tracing::error!("unhandled exception: {}", e);
                    no(e)
                }
            };
            tracing::debug!("final resp = {}", serde_json::to_value(&converted)?);
            Ok::<ServerResponse, Error>(converted)
        }
    }))
    .await
}
