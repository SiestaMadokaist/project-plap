use aws_sdk_dynamodb::Client;

use crate::{
    application::ports::repository::container::{AllRepos, RepositoryContainer},
    config::env::Env,
    infras::repos::dynamo::{
        agent_command::DDBAgentCommandRepository, translation::DDBTranslationRepository,
        user::DDBUserRepository,
    },
};

pub struct DynamoRepositoryContainer {
    translation: DDBTranslationRepository,
    user: DDBUserRepository,
    agent_command: DDBAgentCommandRepository,
}

impl DynamoRepositoryContainer {
    pub fn new(client: &Client, env: &Env) -> Self {
        Self {
            translation: DDBTranslationRepository::new(
                client.clone(),
                env.translation_table.clone(),
            ),
            user: DDBUserRepository::new(client.clone(), env.user_table.clone()),
            agent_command: DDBAgentCommandRepository::new(
                client.clone(),
                env.agent_command_table.clone(),
            ),
        }
    }
}

impl RepositoryContainer for DynamoRepositoryContainer {
    type Translation = DDBTranslationRepository;
    type User = DDBUserRepository;
    type AgentCommand = DDBAgentCommandRepository;
    fn translation(&self) -> &Self::Translation {
        return &self.translation;
    }

    fn user(&self) -> &Self::User {
        return &self.user;
    }

    fn agent_command(&self) -> &Self::AgentCommand {
        return &self.agent_command;
    }
}

impl AllRepos for DynamoRepositoryContainer {}
