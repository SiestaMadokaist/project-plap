use std::rc::Rc;

use aws_sdk_dynamodb::Client;
use serde::{Deserialize, Serialize};

use crate::{
    application::ports::repository::container::{AllRepos, RepositoryContainer},
    infras::repos::{
        dynamo::{
            agent_command::DDBAgentCommandRepository, hotreload::DDBHotReloadRepository,
            translation::DDBTranslationRepository, user::DDBUserRepository,
        },
        prompts::PromptRepository,
    },
    pkg::{enums::stage::Stage, macros::displayable},
};

pub struct LambdaRepos {
    translation: DDBTranslationRepository,
    user: DDBUserRepository,
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
    Translations,
    Users,
    AgentCommands,
    HotReloads,
}
displayable!(TableName);

impl LambdaRepos {
    pub fn rc(client: &Client, stage: Stage) -> Rc<Self> {
        Rc::new(Self::new(client, stage))
    }

    fn to_table(stage: Stage, name: TableName) -> String {
        let v: Vec<String> = vec![stage.into(), name.into()];
        v.join("-")
    }

    pub fn new(client: &Client, stage: Stage) -> Self {
        Self {
            translation: DDBTranslationRepository::new(
                client.clone(),
                Self::to_table(stage, TableName::Translations),
            ),
            user: DDBUserRepository::new(client.clone(), Self::to_table(stage, TableName::Users)),
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

impl RepositoryContainer for LambdaRepos {
    type Translation = DDBTranslationRepository;
    type User = DDBUserRepository;
    type AgentCommand = DDBAgentCommandRepository;
    type HotReload = DDBHotReloadRepository;
    type PromptHistory = PromptRepository;
    fn translation(&self) -> &Self::Translation {
        &self.translation
    }

    fn user(&self) -> &Self::User {
        &self.user
    }

    fn agent_command(&self) -> &Self::AgentCommand {
        &self.agent_command
    }

    fn hotreload(&self) -> &Self::HotReload {
        &self.hotreload
    }

    fn prompt(&self) -> &Self::PromptHistory {
        &self.prompt
    }
}

impl AllRepos for LambdaRepos {}
