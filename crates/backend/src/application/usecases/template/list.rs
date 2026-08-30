use domain::{ctx::Context, storyline::StoryId};
use dto::resources::list::{ListMeta, ListResponse};
use pkg::trait_repos;

use crate::application::ports::{
    repository::{container::HasStoryTemplate, story::StoryTemplateRepository},
    usecase::UsecaseAPI,
};

// trait_clients!(TemplateWriteC)
trait_repos!(TemplateListRepo, HasStoryTemplate);
pub struct TemplateListSvc<R: TemplateListRepo> {
    repos: R,
    ctx: Context,
}

impl<R: TemplateListRepo> TemplateListSvc<R> {
    pub fn new(repos: R, ctx: Context) -> Self {
        Self { repos, ctx }
    }
}

impl<R: TemplateListRepo> UsecaseAPI<ListResponse<StoryId>> for TemplateListSvc<R> {
    async fn exec(&self) -> Result<ListResponse<StoryId>, domain::errors::DomainError> {
        let repo = self.repos.story_template();
        let auth = self.ctx.auth();
        let stories = repo.list(&auth.username).await?;
        let meta = ListMeta::default();
        Ok(ListResponse::new(stories, meta))
    }
}
