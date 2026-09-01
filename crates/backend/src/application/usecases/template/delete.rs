use domain::{ctx::Context, storyline::StoryId};
use dto::resources::{list::ListResponse, templates::DeleteTemplatePayload};
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

impl<'a, R: TemplateListRepo> UsecaseAPI<ListResponse<StoryId>> for TemplateDeleteSvc<'a, R> {
    async fn exec(&self) -> Result<ListResponse<StoryId>, domain::errors::DomainError> {
        let repo = self.repos.story_template();
        let auth = self.ctx.auth();
        let username = &auth.username;
        repo.delete(username, &self.payload.0).await?;
        Ok(ListResponse::simple(vec![]))
    }
}
