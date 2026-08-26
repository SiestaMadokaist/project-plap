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
mod resp;
mod routes;

use bootstrap::{client::ApiClients, repo::ApiRepos};
use env::ApiEnv;

use crate::{
    resp::{no, yes, ServerResponse},
    routes::{ApiEvent, HttpEvent, RouteId},
};

fn routes() -> Router<RouteId> {
    let mut router = Router::new();
    let expectation = "must be valid route";
    router
        .insert(RouteId::ListModels.to_string(), RouteId::ListModels)
        .expect(expectation);
    router
        .insert(
            RouteId::AgentCommandFetchModel,
            RouteId::AgentCommandFetchModel,
        )
        .expect(expectation);
    router
}

async fn handler(
    repos: Rc<ApiRepos>,
    clients: Rc<ApiClients>,
    router: Rc<Router<RouteId>>,
    event: HttpEvent,
) -> Result<ServerResponse, DomainError> {
    let path = event.path();
    let route = router.at(path);
    let resp: Result<dto::response::Response<serde_json::Value>, DomainError> = match route {
        Err(x) => Err(DomainError::Prerequisite(x.to_string())),
        Ok(matched) => match matched.value {
            RouteId::ListModels => GetList::new(clients.clone(), event.body()?.try_into()?)
                .exec()
                .await
                .to_result(),
            RouteId::AgentCommandFetchModel => {
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

    run(service_fn(move |event: LambdaEvent<ApiEvent>| {
        let r = repo.clone();
        let c = client.clone();
        let rt = router.clone();
        let http_event = HttpEvent(event);
        async move {
            let handled: Result<ServerResponse, DomainError> = handler(r, c, rt, http_event).await;
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
