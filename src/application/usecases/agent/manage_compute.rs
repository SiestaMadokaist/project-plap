use std::rc::Rc;

use crate::{
    application::ports::clients::{
        self,
        compute::{ComputeEngine, ComputeEngines},
    },
    domain::commands::compute::{ComputeArgs, ComputeCommand, ComputeError},
    pkg::macros::trait_clients,
};

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

    pub async fn exec(&self) -> anyhow::Result<()> {
        let engines = self.clients.engines();
        let command = &self.args.command;
        let id = &self.args.instance_id;
        let region = &self.args.region;
        let opt_engine = engines.get(region);
        if opt_engine.is_none() {
            let err = ComputeError::InvalidRegion(region.into());
            return Err(err.into());
        }
        let engine = opt_engine.expect("msg");
        match command {
            ComputeCommand::Terminate => engine.terminate(id).await?,
            ComputeCommand::Reboot => engine.reboot(id).await?,
            ComputeCommand::Stop => engine.stop(id).await?,
        };
        Ok(())
    }
}
