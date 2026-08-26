use std::rc::Rc;

use crate::application::ports::clients::{
    self,
    compute::{ComputeEngine, ComputeEngines},
};
use domain::{
    commands::compute::{ComputeArgs, ComputeCommand},
    errors::DomainError,
};
use pkg::macros::trait_clients;

trait_clients!(
    ManageComputeClients,
    clients::container::HasEngines,
    clients::container::HasNotification
);

pub struct ManageCompute<C: ManageComputeClients> {
    clients: Rc<C>,
    args: ComputeArgs,
}

impl<C: ManageComputeClients> ManageCompute<C> {
    pub fn new(clients: Rc<C>, args: ComputeArgs) -> Self {
        Self { clients, args }
    }

    pub async fn exec(&self) -> Result<(), DomainError> {
        let engines = self.clients.engines();
        let command = &self.args.command;
        let id = &self.args.instance_id;
        let region = &self.args.region;
        let opt_engine = engines.get(region);
        let engine = opt_engine.ok_or(DomainError::InvalidRegion(region.to_string()))?;
        match command {
            ComputeCommand::Terminate => engine.terminate(id).await?,
            ComputeCommand::Reboot => engine.reboot(id).await?,
            ComputeCommand::Stop => engine.stop(id).await?,
        };
        Ok(())
    }
}
