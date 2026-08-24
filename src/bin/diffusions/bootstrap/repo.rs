use std::rc::Rc;

use aws_sdk_dynamodb::Client;

use rust_api::{
    application::ports::repository::container::{HasAgentCommand, HasHotReload, HasPromptHistory},
    constant::ddb::DDBTable::{self},
    infras::repos::{
        dynamo::{agent_command::DDBAgentCommandRepository, hotreload::DDBHotReloadRepository},
        prompts::PromptRepository,
    },
    pkg::enums::stage::Stage,
};

pub struct EC2DiffusionRepo {
    agent_command: DDBAgentCommandRepository,
    hotreload: DDBHotReloadRepository,
    prompt: PromptRepository,
}

impl EC2DiffusionRepo {
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
                DDBTable::HotReloads.table_name(stage),
            ),
            prompt: PromptRepository::default(),
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
