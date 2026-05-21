use std::future::Future;

use crate::domain::user::{User, UserId};

#[derive(Debug, thiserror::Error)]
pub enum UserRepoError {
    #[error("user not found: {0}")]
    NotFound(UserId),
    #[error("database error: {0}")]
    Database(String),
}

pub trait UserRepository {
    fn get(&self, id: &UserId) -> impl Future<Output = Result<User, UserRepoError>> + Send;
    fn put(&self, user: &User) -> impl Future<Output = Result<(), UserRepoError>> + Send;
    fn delete(&self, id: &UserId) -> impl Future<Output = Result<(), UserRepoError>> + Send;
}
