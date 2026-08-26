use std::rc::Rc;

use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use matchit::Router;
use backend::{
    application::usecases::hq::commands::cp_model::CPModel,
    application::usecases::hq::models::list::GetList,
};
use domain::errors::DomainError;

mod bootstrap;
mod env;
mod routes;

use bootstrap::{client::ApiClients, repo::ApiRepos};
use env::ApiEnv;

use crate::routes::{err_response, json_response, ApiEvent, ApiResponse, HttpEvent, RouteId};

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
) -> Result<ApiResponse, DomainError> {
    let path = event.path();
    let resp = match router.at(path) {
        Ok(matched) => {
            let resp: Result<serde_json::Value, DomainError> = match matched.value {
                RouteId::ListModels => {
                    GetList::new(clients.clone(), event.body()?.try_into()?)
                        .exec()
                        .await
                }
                RouteId::AgentCommandFetchModel => {
                    CPModel::new(repos.clone(), event.body()?.try_into()?)
                        .exec()
                        .await
                }
                _ => Err(DomainError::NotImplemented),
            };
            json_response(200, resp?.to_string())
        }
        Err(_) => json_response(404, r#"{"error":"not found"}"#),
    };
    Ok(resp)
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
            let handled: Result<ApiResponse, DomainError> = handler(r, c, rt, http_event).await;
            let converted: ApiResponse = match handled {
                Ok(x) => x,
                Err(e) => {
                    tracing::error!("unhandled exception: {}", e);
                    err_response(&e)
                }
            };
            tracing::debug!("final resp = {}", serde_json::to_value(&converted)?);
            Ok::<ApiResponse, Error>(converted)
        }
    }))
    .await
}
