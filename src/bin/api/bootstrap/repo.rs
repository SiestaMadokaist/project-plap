use std::rc::Rc;

use aws_sdk_dynamodb::Client;
use rust_api::{
    application::ports::repository::container::{HasAgentCommand, HasHotReload},
    infras::repos::dynamo::{
        agent_command::DDBAgentCommandRepository, hotreload::DDBHotReloadRepository,
    },
    pkg::{enums::stage::Stage, macros::displayable},
};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TableName {
    AgentCommands,
    HotReloads,
}
displayable!(TableName);

pub struct ApiRepos {
    agent_command: DDBAgentCommandRepository,
    hotreload: DDBHotReloadRepository,
}

impl ApiRepos {
    pub fn rc(client: &Client, stage: Stage) -> Rc<Self> {
        Rc::new(Self::new(client, stage))
    }

    fn to_table(stage: Stage, name: TableName) -> String {
        let v: Vec<String> = vec![stage.into(), name.into()];
        v.join("-")
    }

    pub fn new(client: &Client, stage: Stage) -> Self {
        Self {
            agent_command: DDBAgentCommandRepository::new(
                client.clone(),
                Self::to_table(stage, TableName::AgentCommands),
            ),
            hotreload: DDBHotReloadRepository::new(
                client.clone(),
                Self::to_table(stage, TableName::HotReloads),
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
