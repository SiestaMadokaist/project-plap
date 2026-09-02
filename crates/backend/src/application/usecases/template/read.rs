use domain::{ctx::Context, storyline::Storyline};
use dto::resources::templates::ReadTemplatePayload;
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

impl<'a, R: TemplateReadRepo> UsecaseAPI<Storyline> for TemplateReadSvc<'a, R> {
    async fn exec(&self) -> Result<Storyline, domain::errors::DomainError> {
        let repo = self.repos.story_template();
        let auth = self.ctx.auth();
        repo.get(&auth.username, &self.payload.0).await
    }
}
