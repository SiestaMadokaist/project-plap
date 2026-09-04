use crate::application::ports::{
    clients::{
        self,
        compute::{ComputeEngine, ComputeEngines},
    },
    usecase::UsecaseAPI,
};
use domain::{commands::compute::ComputeCommand, errors::DomainError};
use dto::resources::computes::{ComputeControlPayload, ComputeDTO};
use pkg::macros::trait_clients;

trait_clients!(ManageComputeClients, clients::container::HasEngines);

pub struct ManageCompute<'a, C: ManageComputeClients> {
    clients: &'a C,
    payload: ComputeControlPayload,
}

impl<'a, C: ManageComputeClients> ManageCompute<'a, C> {
    pub fn new(clients: &'a C, payload: ComputeControlPayload) -> Self {
        Self { clients, payload }
    }
}

impl<'a, C: ManageComputeClients> UsecaseAPI<ComputeDTO> for ManageCompute<'a, C> {
    async fn exec(&self) -> Result<ComputeDTO, DomainError> {
        let engines = self.clients.engines();
        let args = &self.payload.0;
        let command = &args.command;
        let id = &args.instance_id;
        let region = &args.region;
        let opt_engine = engines.get(region);
        let engine = opt_engine.ok_or_else(|| DomainError::InvalidRegion(region.to_string()))?;
        match command {
            ComputeCommand::Start => engine.start(id).await?,
            ComputeCommand::Terminate => engine.terminate(id).await?,
            ComputeCommand::Reboot => engine.reboot(id).await?,
            ComputeCommand::Stop => engine.stop(id).await?,
        };
        let instance = engine
            .list()
            .await?
            .into_iter()
            .find(|i| &i.id == id)
            .ok_or(DomainError::NotFound)?;
        Ok(ComputeDTO(instance))
    }
}
