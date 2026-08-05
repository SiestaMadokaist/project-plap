use std::rc::Rc;

use crate::{
    application::{ports::clients::compute::ComputeClient, usecases::agent::traits::AgentClients},
    domain::commands::compute::ComputeArgs,
};

pub struct ManageCompute<C: AgentClients> {
    clients: Rc<C>,
    args: ComputeArgs,
}

impl<C: AgentClients> ManageCompute<C> {
    pub fn new(clients: Rc<C>, args: ComputeArgs) -> Self {
        Self { clients, args }
    }

    async fn launch(&self) -> anyhow::Result<()> {
        let compute = self.clients.compute();
        let result = compute.launch().await?;
        todo!();
    }

    pub async fn exec(&self) -> anyhow::Result<()> {
        todo!();
    }
}
