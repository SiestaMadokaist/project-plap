use domain::{ctx::Context, errors::DomainError};
use dto::resources::computes::{ComputeDTO, LaunchPayload};
use pkg::{trait_clients, trait_repos};

use crate::application::ports::{
    clients::{
        self,
        compute::{ComputeEngine, ComputeEngines},
    },
    repository::{container::HasHotReload, hot_reload::HotReloadRepository},
    usecase::UsecaseAPI,
};

trait_clients!(LaunchComputeClients, clients::container::HasEngines);
trait_repos!(LaunchComputeRepos, HasHotReload);

pub struct LaunchCompute<'a, C: LaunchComputeClients, R: LaunchComputeRepos> {
    clients: &'a C,
    repos: &'a R,
    ctx: &'a Context,
    payload: LaunchPayload,
}

impl<'a, C: LaunchComputeClients, R: LaunchComputeRepos> LaunchCompute<'a, C, R> {
    pub fn new(clients: &'a C, repos: &'a R, ctx: &'a Context, payload: LaunchPayload) -> Self {
        Self {
            clients,
            repos,
            ctx,
            payload,
        }
    }
}

impl<'a, C: LaunchComputeClients, R: LaunchComputeRepos> UsecaseAPI<ComputeDTO>
    for LaunchCompute<'a, C, R>
{
    async fn exec(&self) -> Result<ComputeDTO, DomainError> {
        let username = &self.ctx.auth().username;
        let region = self.payload.region;
        let config = self
            .repos
            .hotreload()
            .launch_config(username, &region)
            .await?;
        let engine = self
            .clients
            .engines()
            .get(&region)
            .ok_or_else(|| DomainError::InvalidRegion(region.to_string()))?;
        let instance = engine
            .launch(&config, username, &None, self.payload.spot)
            .await?;
        Ok(ComputeDTO(instance))
    }
}
