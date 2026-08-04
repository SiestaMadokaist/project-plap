use std::rc::Rc;

use crate::{
    application::usecases::agent::traits::{AgentClients, AgentRepos},
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

    pub async fn exec(&self) -> anyhow::Result<()> {
        todo!();
    }
}
