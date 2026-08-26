use std::rc::Rc;

use aws_sdk_dynamodb::Client;
use backend::{
    application::ports::repository::container::{HasAgentCommand, HasHotReload},
    constant::ddb::DDBTable,
    infras::repos::dynamo::{
        agent_command::DDBAgentCommandRepository, hotreload::DDBHotReloadRepository,
    },
};
use pkg::enums::stage::Stage;

pub struct ApiRepos {
    agent_command: DDBAgentCommandRepository,
    hotreload: DDBHotReloadRepository,
}

impl ApiRepos {
    pub fn rc(client: &Client, stage: Stage) -> Rc<Self> {
        Rc::new(Self::new(client, stage))
    }

    pub fn new(client: &Client, stage: Stage) -> Self {
        Self {
            agent_command: DDBAgentCommandRepository::new(
                client.clone(),
                DDBTable::AgentCommands.table_name(stage),
            ),
            hotreload: DDBHotReloadRepository::new(
                client.clone(),
                DDBTable::AgentCommands.table_name(stage),
            ),
        }
    }
}

impl HasAgentCommand for ApiRepos {
    type AgentCommand = DDBAgentCommandRepository;
    fn agent_command(&self) -> &Self::AgentCommand {
        &self.agent_command
    }
}

impl HasHotReload for ApiRepos {
    type HotReload = DDBHotReloadRepository;
    fn hotreload(&self) -> &Self::HotReload {
        &self.hotreload
    }
}
