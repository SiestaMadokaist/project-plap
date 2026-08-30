use domain::{
    errors::DomainError,
    storyline::{StoryId, Storyline},
};
use pkg::auth::claims::Username;

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait StoryTemplateRepository {
    async fn list(&self, username: &Username) -> Result<Vec<StoryId>, DomainError>;

    async fn get(&self, id: &StoryId) -> Result<Storyline, DomainError>;

    async fn write(&self, payload: &Storyline) -> Result<Storyline, DomainError>;
}
