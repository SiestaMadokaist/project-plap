use std::rc::Rc;

use backend::application::{
    ports::{
        clients::{authorizer::Authorizer, container::HasAuthValidator},
        usecase::UsecaseAPI,
    },
    routes::{
        authorized::AuthorizedRoute::{self, TemplateList},
        public::PublicRoute::{self},
    },
    usecases::{
        hq::{
            commands::cp_model::CPModel,
            health::Healthcheck,
            models::list::GetList,
            users::{challenge::GetChallenge, submit_answer::SubmitAnswer},
        },
        template::list::TemplateListSvc,
    },
};
use domain::{ctx, errors::DomainError};
use dto::response::{Placeholder, ToResp};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use matchit::Router;

mod bootstrap;
mod env;
mod req;
mod resp;

use bootstrap::{client::ApiClients, repo::ApiRepos};
use env::ApiEnv;
use pkg::{auth::claims::JWT, id::TraceId};
use tracing_subscriber::layer::Context;

use crate::{
    req::{ApiEvent, HttpEvent},
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
            AuthorizedRoute::AgentModelCP.to_string(),
            AuthorizedRoute::AgentModelCP,
        )
        .expect(expectation);
    router
        .insert(
            AuthorizedRoute::TemplateList.to_string(),
            AuthorizedRoute::TemplateList,
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
        .insert(PublicRoute::Health.to_string(), PublicRoute::Health)
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
            GetChallenge::new(clients.as_ref(), repos.as_ref(), event.body()?.try_into()?)
                .exec()
                .await
                .to_result()
        }
        PublicRoute::SubmitAnswer => {
            SubmitAnswer::new(clients.as_ref(), repos.as_ref(), event.body()?.try_into()?)
                .exec()
                .await
                .to_result()
        }
        PublicRoute::Health => Healthcheck::default().exec().await.to_result(),
    };
    resp.and_then(yes)
}

async fn init_ctx(clients: &ApiClients, token: JWT) -> Result<ctx::Context, DomainError> {
    let claims = clients.authorizer().validate(token).await?;
    let ctx = ctx::Context::new(claims, None);
    Ok(ctx)
}

async fn handle_authorized(
    repos: Rc<ApiRepos>,
    clients: Rc<ApiClients>,
    router: Rc<Router<AuthorizedRoute>>,
    event: HttpEvent,
) -> Result<ServerResponse, DomainError> {
    let path = event.path().to_string();
    let ctx = init_ctx(&clients, event.authorization()?).await?;
    let route = router.at(&path);
    let resp: Result<dto::response::Response<serde_json::Value>, DomainError> = match route {
        Err(x) => Err(DomainError::Prerequisite(x.to_string())),
        Ok(matched) => match matched.value {
            AuthorizedRoute::ListModels => {
                GetList::new(clients.as_ref(), event.body()?.try_into()?)
                    .exec()
                    .await
                    .to_result()
            }
            AuthorizedRoute::AgentModelCP => {
                CPModel::new(repos.as_ref(), event.body()?.try_into()?)
                    .exec()
                    .await
                    .to_result()
            }
            // AuthorizedRoute::TemplateList => {
            //     let svc = TemplateListSvc::new(repos.clone(), ctx);
            //     let result = svc.exec();
            // }
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
            // check public first because authorized might throw error if unauthorized
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
            Ok::<ServerResponse, Error>(converted)
        }
    }))
    .await
}
