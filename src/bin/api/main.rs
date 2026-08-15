use std::rc::Rc;

use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use matchit::Router;
use rust_api::{
    application::usecases::hq::models::list::GetListModel, domain::errors::DomainError,
};

mod bootstrap;
mod env;
mod routes;

use bootstrap::{client::ApiClients, repo::ApiRepos};
use env::ApiEnv;

use crate::routes::{err_response, json_response, ApiEvent, ApiResponse, HttpEvent, RouteId};

fn routes() -> Router<RouteId> {
    let mut router = Router::new();
    router
        .insert("/", RouteId::ListModels)
        .expect("valid route");
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
                    GetListModel::new(clients.clone(), event.body()?.try_into()?)
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
    rust_api::init_env();
    rust_api::init_tracing();

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
                    err_response(e)
                }
            };
            Ok::<ApiResponse, Error>(converted)
        }
    }))
    .await
}
