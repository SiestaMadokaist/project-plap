use std::rc::Rc;

use backend::application::{
    ports::usecase::UsecaseAPI,
    routes::public::PublicRoute,
    usecases::hq::{
        health::Healthcheck,
        users::{challenge::GetChallenge, submit_answer::SubmitAnswer},
    },
};
use domain::errors::DomainError;
use dto::response::ToResp;
use matchit::Router;

use crate::{
    bootstrap::{client::ApiClients, repo::ApiRepos},
    http::{
        req::HttpEvent,
        resp::{yes, ServerResponse},
    },
};

pub fn public_routes() -> Router<PublicRoute> {
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

pub async fn handle_public(
    repos: Rc<ApiRepos>,
    clients: Rc<ApiClients>,
    router: Rc<Router<PublicRoute>>,
    event: HttpEvent,
) -> Result<ServerResponse, DomainError> {
    let path = event.path().to_string();
    let matched = router
        .at(&path)
        .map_err(|e| DomainError::Prerequisite(e.to_string()))?;
    let resp: Result<dto::response::Response<serde_json::Value>, DomainError> = match matched.value {
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
