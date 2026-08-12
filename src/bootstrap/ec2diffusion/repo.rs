use std::rc::Rc;

use aws_sdk_dynamodb::Client;
use serde::{Deserialize, Serialize};

use crate::{
    application::ports::repository::container::{HasAgentCommand, HasHotReload, HasPromptHistory},
    infras::repos::{
        dynamo::{agent_command::DDBAgentCommandRepository, hotreload::DDBHotReloadRepository},
        prompts::PromptRepository,
    },
    pkg::{enums::stage::Stage, macros::displayable},
};

pub struct EC2DiffusionRepo {
    agent_command: DDBAgentCommandRepository,
    hotreload: DDBHotReloadRepository,
    prompt: PromptRepository,
}

/**
 * @todo:
 * this introduce a name change for table
 * eg: agent_commands -> agentcommands, unless
 */
#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TableName {
    AgentCommands,
    HotReloads,
}
displayable!(TableName);

impl EC2DiffusionRepo {
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
            prompt: PromptRepository::new(),
        }
    }
}

impl HasAgentCommand for EC2DiffusionRepo {
    type AgentCommand = DDBAgentCommandRepository;
    fn agent_command(&self) -> &Self::AgentCommand {
        &self.agent_command
    }
}

impl HasHotReload for EC2DiffusionRepo {
    type HotReload = DDBHotReloadRepository;
    fn hotreload(&self) -> &Self::HotReload {
        &self.hotreload
    }
}

impl HasPromptHistory for EC2DiffusionRepo {
    type PromptHistory = PromptRepository;
    fn prompt(&self) -> &Self::PromptHistory {
        &self.prompt
    }
}
