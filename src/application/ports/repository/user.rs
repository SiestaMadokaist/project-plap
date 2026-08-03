use crate::{
    application::ports::repository::error::RepositoryError,
    domain::user::{User, UserId},
};

pub type UserError = RepositoryError<UserId>;

#[allow(async_fn_in_trait)]
pub trait UserRepository {
    async fn get(&self, id: &UserId) -> Result<User, UserError>;
    async fn put(&self, user: &User) -> Result<(), UserError>;
    async fn delete(&self, id: &UserId) -> Result<(), UserError>;
}
