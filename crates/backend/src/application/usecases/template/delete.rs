use domain::{ctx::Context, errors::DomainError};
use dto::resources::{
    list::ListResponse,
    templates::{DeleteTemplatePayload, ListStoryTemplate},
};
use pkg::trait_repos;

use crate::application::ports::{
    repository::{container::HasStoryTemplate, story::StoryTemplateRepository},
    usecase::UsecaseAPI,
};

trait_repos!(TemplateListRepo, HasStoryTemplate);
pub struct TemplateDeleteSvc<'a, R: TemplateListRepo> {
    repos: &'a R,
    ctx: &'a Context,
    payload: DeleteTemplatePayload,
}

impl<'a, R: TemplateListRepo> TemplateDeleteSvc<'a, R> {
    pub fn new(repos: &'a R, ctx: &'a Context, payload: DeleteTemplatePayload) -> Self {
        Self {
            repos,
            ctx,
            payload,
        }
    }
}

impl<'a, R: TemplateListRepo> UsecaseAPI<ListStoryTemplate> for TemplateDeleteSvc<'a, R> {
    async fn exec(&self) -> Result<ListStoryTemplate, domain::errors::DomainError> {
        let repo = self.repos.story_template();
        let claims = self
            .ctx
            .auth()
            .ok_or(DomainError::NotAllowed("unauthorized".into()))?;
        let username = &claims.username;
        repo.delete(username, &self.payload.0).await?;
        Ok(ListResponse::simple(vec![]))
    }
}
