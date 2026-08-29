use std::rc::Rc;

use backend::application::{
    ports::{
        clients::{authorizer::Authorizer, container::HasAuthValidator},
        usecase::UsecaseAPI,
    },
    usecases::hq::{
        commands::cp_model::CPModel,
        models::list::GetList,
        users::{challenge::GetChallenge, submit_answer::SubmitAnswer},
    },
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

use crate::{
    req::{ApiEvent, AuthorizedRoute, HttpEvent, PublicRoute},
    resp::{no, yes, ServerResponse},
};

fn authorized_routes() -> Router<AuthorizedRoute> {
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
            AuthorizedRoute::AgentCommandFetchModel.to_string(),
            AuthorizedRoute::AgentCommandFetchModel,
        )
        .expect(expectation);
    router
}

fn public_routes() -> Router<PublicRoute> {
    let mut router = Router::new();
    let expectation = "must be valid route";
    router
        .insert(
            PublicRoute::GetChallenge.to_string(),
            PublicRoute::GetChallenge,
        )
        .expect(expectation);
    router
        .insert(
            PublicRoute::SubmitAnswer.to_string(),
            PublicRoute::SubmitAnswer,
        )
        .expect(expectation);
    router
}

async fn handle_public(
    repos: Rc<ApiRepos>,
    clients: Rc<ApiClients>,
    router: Rc<Router<PublicRoute>>,
    event: HttpEvent,
) -> Result<ServerResponse, DomainError> {
    let path = event.path().to_string();
    let matched = router
        .at(&path)
        .map_err(|e| DomainError::Prerequisite(e.to_string()))?;
    let resp: Result<dto::response::Response<serde_json::Value>, DomainError> = match matched.value
    {
        PublicRoute::GetChallenge => {
            GetChallenge::new(clients.clone(), repos.clone(), event.body()?.try_into()?)
                .exec()
                .await
                .to_result()
        }
        PublicRoute::SubmitAnswer => {
            SubmitAnswer::new(clients.clone(), repos.clone(), event.body()?.try_into()?)
                .exec()
                .await
                .to_result()
        }
    };
    resp.and_then(yes)
}

async fn handle_authorized(
    repos: Rc<ApiRepos>,
    clients: Rc<ApiClients>,
    router: Rc<Router<AuthorizedRoute>>,
    event: HttpEvent,
) -> Result<ServerResponse, DomainError> {
    // gate: a well-formed, unexpired, untampered token is all a route needs - the
    // usecases don't inspect the claims themselves.
    let token = event.authorization()?;
    let _claims = clients.authorizer().validate(token).await?;

    let path = event.path().to_string();
    let route = router.at(&path);
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
    let authorized_router = Rc::new(authorized_routes());
    let public_router = Rc::new(public_routes());

    run(service_fn(move |event: LambdaEvent<ApiEvent>| {
        let r = repo.clone();
        let c = client.clone();
        let public_rt = public_router.clone();
        let authorized_rt = authorized_router.clone();
        let http_event = HttpEvent(event);
        async move {
            let path = http_event.path().to_string();
            let handled: Result<ServerResponse, DomainError> = if public_rt.at(&path).is_ok() {
                handle_public(r, c, public_rt, http_event).await
            } else {
                handle_authorized(r, c, authorized_rt, http_event).await
            };
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
