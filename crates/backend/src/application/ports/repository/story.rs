use domain::{
    errors::DomainError,
    storyline::{StoryTemplateId, Storyline},
};
use pkg::auth::claims::Username;

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait StoryTemplateRepository {
    async fn list(&self, owner: &Username) -> Result<Vec<StoryTemplateId>, DomainError>;

    async fn get(&self, owner: &Username, id: &StoryTemplateId) -> Result<Storyline, DomainError>;

    async fn write(&self, owner: &Username, payload: &Storyline) -> Result<Storyline, DomainError>;

    async fn delete(&self, owner: &Username, id: &StoryTemplateId) -> Result<(), DomainError>;
}
