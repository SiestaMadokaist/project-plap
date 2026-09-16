use domain::ctx::Context;
use dto::resources::templates::{ReadTemplatePayload, ReadTemplateResponse};
use pkg::trait_repos;

use crate::application::ports::{
    repository::{container::HasStoryTemplate, story::StoryTemplateRepository},
    usecase::UsecaseAPI,
};

trait_repos!(TemplateReadRepo, HasStoryTemplate);
pub struct TemplateReadSvc<'a, R: TemplateReadRepo> {
    repos: &'a R,
    ctx: &'a Context,
    payload: ReadTemplatePayload,
}

impl<'a, R: TemplateReadRepo> TemplateReadSvc<'a, R> {
    pub fn new(repos: &'a R, ctx: &'a Context, payload: ReadTemplatePayload) -> Self {
        Self {
            repos,
            ctx,
            payload,
        }
    }
}

impl<'a, R: TemplateReadRepo> UsecaseAPI<ReadTemplateResponse> for TemplateReadSvc<'a, R> {
    async fn exec(&self) -> Result<ReadTemplateResponse, domain::errors::DomainError> {
        let repo = self.repos.story_template();
        let auth = self.ctx.auth();
        let storyline = repo.get(&auth.username, &self.payload.0).await?;
        Ok(ReadTemplateResponse(storyline))
    }
}
