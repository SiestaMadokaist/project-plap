use domain::{
    errors::DomainError,
    storylines::{Storyline, StorylineId},
};

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait StorylineRepository {
    async fn list(&self) -> Vec<StorylineId>;

    async fn get(&self, id: &StorylineId) -> Result<Storyline, DomainError>;
}
