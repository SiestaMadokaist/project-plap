use std::rc::Rc;

use domain::storyline::{RawTemplate, Storyline};
use dto::{resources::templates::WriteTemplatePayload, response::Placeholder};
use pkg::trait_repos;

use crate::application::ports::{
    repository::{container::HasStoryTemplate, story::StoryTemplateRepository},
    usecase::UsecaseAPI,
};

// trait_clients!(TemplateWriteC)
trait_repos!(TemplateWriteRepo, HasStoryTemplate);
pub struct TemplateWrite<'a, R: TemplateWriteRepo> {
    repos: Rc<R>,
    payload: &'a WriteTemplatePayload,
}

impl<'a, R: TemplateWriteRepo> UsecaseAPI<Placeholder> for TemplateWrite<'a, R> {
    async fn exec(&self) -> Result<Placeholder, domain::errors::DomainError> {
        let repo = self.repos.story_template();
        repo.write(&self.payload.0).await?;
        Ok(Placeholder(200))
    }
}
