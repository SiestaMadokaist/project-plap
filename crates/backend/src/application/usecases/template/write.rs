use domain::ctx::Context;
use dto::{resources::templates::WriteTemplatePayload, response::Placeholder};
use pkg::trait_repos;

use crate::application::ports::{
    repository::{container::HasStoryTemplate, story::StoryTemplateRepository},
    usecase::UsecaseAPI,
};

// trait_clients!(TemplateWriteC)
trait_repos!(TemplateWriteRepo, HasStoryTemplate);
pub struct TemplateWriteSvc<'a, R: TemplateWriteRepo> {
    repos: &'a R,
    ctx: &'a Context,
    payload: WriteTemplatePayload,
}

impl<'a, R: TemplateWriteRepo> TemplateWriteSvc<'a, R> {
    pub fn new(repos: &'a R, ctx: &'a Context, payload: WriteTemplatePayload) -> Self {
        Self {
            repos,
            ctx,
            payload,
        }
    }
}

impl<'a, R: TemplateWriteRepo> UsecaseAPI<Placeholder> for TemplateWriteSvc<'a, R> {
    async fn exec(&self) -> Result<Placeholder, domain::errors::DomainError> {
        let repo = self.repos.story_template();
        let auth = self.ctx.auth();
        repo.write(&auth.username, &self.payload.0).await?;
        Ok(Placeholder(200))
    }
}
