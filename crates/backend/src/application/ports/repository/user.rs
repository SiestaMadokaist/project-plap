use crate::application::ports::repository::error::RepositoryError;
use domain::user::{User, UserId};
use pkg::{auth::ecdsa::AddressETH, types::time::Timestamp};

pub type UserError = RepositoryError<UserId>;

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait UserRepository {
    /// Look up by `username` (the table's partition key).
    async fn get(&self, id: &UserId) -> Result<User, UserError>;
    async fn put(&self, user: &User) -> Result<(), UserError>;
    async fn delete(&self, id: &UserId) -> Result<(), UserError>;

    /// Look up by wallet address via the `address` GSI.
    async fn find(&self, address: &AddressETH) -> Result<User, UserError>;

    /// Record a successful login for the wallet at `address`, stamping `last_login = iat`.
    ///
    /// Fails with `NotFound` if no user has that address, and `Conflict` if the account
    /// isn't activated or if `iat` is not strictly newer than the stored `last_login`
    /// (the latter enforced as a DynamoDB condition, so concurrent logins can't race it).
    async fn login(&self, address: &AddressETH, iat: Timestamp) -> Result<User, UserError>;
}
