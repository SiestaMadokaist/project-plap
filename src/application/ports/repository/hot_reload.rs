use crate::{
    application::ports::repository::error::RepositoryError,
    domain::{hot_reload::DiffusionConfigDomain, user::UserId},
};

pub type HotReloadError = RepositoryError<UserId>;
#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait HotReloadRepository {
    async fn diffusion_config(&self, id: &UserId) -> Result<DiffusionConfigDomain, HotReloadError>;
    // async fn arkham_config(&self, id: &UserId) ->
}
