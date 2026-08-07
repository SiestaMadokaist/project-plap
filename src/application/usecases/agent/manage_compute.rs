use std::rc::Rc;

use crate::{
    application::{ports::clients::compute::ComputeClient, usecases::agent::traits::AgentClients},
    domain::commands::compute::{ComputeArgs, ComputeCommand},
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
        let compute = self.clients.compute();
        let command = &self.args.command;
        let id = &self.args.instance_id;
        match command {
            ComputeCommand::Terminate => compute.terminate(id).await?,
            ComputeCommand::Reboot => compute.reboot(id).await?,
            ComputeCommand::Stop => compute.stop(id).await?,
        };
        Ok(())
    }
}
