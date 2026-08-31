use domain::{ctx::Context, storyline::StoryId};
use dto::resources::list::ListResponse;
use pkg::trait_repos;

use crate::application::ports::{
    repository::{container::HasStoryTemplate, story::StoryTemplateRepository},
    usecase::UsecaseAPI,
};

trait_repos!(TemplateListRepo, HasStoryTemplate);
pub struct TemplateListSvc<'a, R: TemplateListRepo> {
    repos: &'a R,
    ctx: &'a Context,
}

impl<'a, R: TemplateListRepo> TemplateListSvc<'a, R> {
    pub fn new(repos: &'a R, ctx: &'a Context) -> Self {
        Self { repos, ctx }
    }
}

impl<'a, R: TemplateListRepo> UsecaseAPI<ListResponse<StoryId>> for TemplateListSvc<'a, R> {
    async fn exec(&self) -> Result<ListResponse<StoryId>, domain::errors::DomainError> {
        let repo = self.repos.story_template();
        let auth = self.ctx.auth();
        let stories = repo.list(&auth.username).await?;
        Ok(ListResponse::simple(stories))
    }
}
