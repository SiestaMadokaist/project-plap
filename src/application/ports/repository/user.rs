use crate::domain::user::{User, UserId};

#[derive(Debug, thiserror::Error)]
pub enum UserRepoError {
    #[error("user not found: {0}")]
    NotFound(UserId),
    #[error("database error: {0}")]
    Database(String),
}

#[warn(async_fn_in_trait)]
pub trait UserRepository {
    async fn get(&self, id: &UserId) -> Result<User, UserRepoError>;
    async fn put(&self, user: &User) -> Result<(), UserRepoError>;
    async fn delete(&self, id: &UserId) -> Result<(), UserRepoError>;
}
