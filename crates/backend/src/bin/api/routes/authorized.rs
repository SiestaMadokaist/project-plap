use std::rc::Rc;

use backend::application::{
    ports::usecase::UsecaseAPI,
    routes::authorized::AuthorizedRoute,
    usecases::{
        hq::{
            commands::cp_model::CPModelSvc,
            models::{list::GetListSvc, preview::PreviewSvc},
        },
        template::{
            delete::TemplateDeleteSvc, list::TemplateListSvc, read::TemplateReadSvc,
            write::TemplateWriteSvc,
        },
    },
};
use domain::{ctx, errors::DomainError};
use dto::response::ToResp;
use matchit::Router;

use crate::{
    bootstrap::{client::ApiClients, repo::ApiRepos},
    http::{
        req::HttpEvent,
        resp::{yes, ServerResponse},
    },
};

pub fn authorized_routes() -> Router<AuthorizedRoute> {
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
            AuthorizedRoute::ModelPreview.to_string(),
            AuthorizedRoute::ModelPreview,
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
        .insert(
            AuthorizedRoute::TemplateWrite.to_string(),
            AuthorizedRoute::TemplateWrite,
        )
        .expect(expectation);
    router
        .insert(
            AuthorizedRoute::TemplateDelete.to_string(),
            AuthorizedRoute::TemplateDelete,
        )
        .expect(expectation);
    router
        .insert(
            AuthorizedRoute::TemplateRead.to_string(),
            AuthorizedRoute::TemplateRead,
        )
        .expect(expectation);
    router
}

pub async fn handle_authorized(
    repos: Rc<ApiRepos>,
    clients: Rc<ApiClients>,
    router: Rc<Router<AuthorizedRoute>>,
    ctx: &ctx::Context,
    event: HttpEvent,
) -> Result<ServerResponse, DomainError> {
    let path = event.path().to_string();
    let route = router.at(&path);
    let payload = event.body()?;
    let resp: Result<dto::response::Response<serde_json::Value>, DomainError> = match route {
        Err(x) => Err(DomainError::Prerequisite(x.to_string())),
        Ok(matched) => match matched.value {
            AuthorizedRoute::ListModels => {
                let svc = GetListSvc::new(clients.as_ref(), payload.try_into()?);
                svc.exec().await.to_result()
            }
            AuthorizedRoute::ModelPreview => {
                let svc = PreviewSvc::new(clients.as_ref(), payload.try_into()?);
                svc.exec().await.to_result()
            }
            AuthorizedRoute::AgentModelCP => {
                let svc = CPModelSvc::new(repos.as_ref(), payload.try_into()?);
                svc.exec().await.to_result()
            }
            AuthorizedRoute::TemplateWrite => {
                let svc = TemplateWriteSvc::new(repos.as_ref(), ctx, payload.try_into()?);
                svc.exec().await.to_result()
            }
            AuthorizedRoute::TemplateDelete => {
                let svc = TemplateDeleteSvc::new(repos.as_ref(), ctx, payload.try_into()?);
                svc.exec().await.to_result()
            }
            AuthorizedRoute::TemplateList => {
                let svc = TemplateListSvc::new(repos.as_ref(), ctx);
                svc.exec().await.to_result()
            }
            AuthorizedRoute::TemplateRead => {
                let svc = TemplateReadSvc::new(repos.as_ref(), ctx, payload.try_into()?);
                svc.exec().await.to_result()
            }
        },
    };
    resp.and_then(yes)
}
