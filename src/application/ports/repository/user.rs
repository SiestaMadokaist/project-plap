use crate::{
    application::ports::repository::error::RepositoryError,
    domain::user::{User, UserId},
};

#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("user not found: {0}")]
    NotFound(UserId),
    #[error(transparent)]
    Repo(#[from] RepositoryError),
}

#[allow(async_fn_in_trait)]
pub trait UserRepository {
    async fn get(&self, id: &UserId) -> Result<User, UserError>;
    async fn put(&self, user: &User) -> Result<(), UserError>;
    async fn delete(&self, id: &UserId) -> Result<(), UserError>;
}
