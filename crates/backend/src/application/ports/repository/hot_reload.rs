use crate::application::ports::repository::error::RepositoryError;
use domain::hot_reload::DiffusionConfigDomain;
use pkg::auth::claims::Username;

pub type HotReloadError = RepositoryError<Username>;
#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait HotReloadRepository {
    async fn diffusion_config(
        &self,
        id: &Username,
    ) -> Result<DiffusionConfigDomain, HotReloadError>;
    // async fn arkham_config(&self, id: &UserId) ->
}
