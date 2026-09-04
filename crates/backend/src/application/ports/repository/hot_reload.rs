use crate::application::ports::repository::error::RepositoryError;
use domain::{
    commands::compute::{ComputeRegion, LaunchConfig},
    errors::DomainError,
    hot_reload::{BillOptimization, HotreloadDomain},
};
use pkg::auth::claims::Username;

pub type HotReloadError = RepositoryError<Username>;
#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait HotReloadRepository {
    async fn set(
        &self,
        username: &Username,
        value: &HotreloadDomain,
    ) -> Result<HotreloadDomain, DomainError>;
    /// The user's launch config for `region` - a user may have one per region,
    /// so both the identity and the target region narrow it down to one.
    async fn launch_config(
        &self,
        username: &Username,
        region: &ComputeRegion,
    ) -> Result<LaunchConfig, HotReloadError>;
    async fn bill_optimization(
        &self,
        username: &Username,
    ) -> Result<BillOptimization, HotReloadError>;
    async fn get(&self, username: &Username) -> Result<Vec<HotreloadDomain>, HotReloadError>;
}
