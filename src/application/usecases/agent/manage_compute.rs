use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::{
    application::{
        ports::clients::compute::{ComputeEngine, ComputeEngines},
        usecases::agent::traits::AgentClients,
    },
    domain::commands::compute::{ComputeArgs, ComputeCommand, ComputeError},
};

pub struct ManageCompute<C: AgentClients> {
    clients: Rc<C>,
    args: ComputeArgs,
}

impl<C: AgentClients> ManageCompute<C> {
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
