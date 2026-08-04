use std::rc::Rc;

use aws_sdk_dynamodb::Client;

use crate::{
    application::ports::repository::container::{AllRepos, RepositoryContainer},
    config::lambda_env::LambdaEnv,
    infras::repos::dynamo::{
        agent_command::DDBAgentCommandRepository, translation::DDBTranslationRepository,
        user::DDBUserRepository,
    },
};

pub struct GeneralRepositories {
    translation: DDBTranslationRepository,
    user: DDBUserRepository,
    agent_command: DDBAgentCommandRepository,
}

impl GeneralRepositories {
    pub fn rc(client: &Client, env: &LambdaEnv) -> Rc<Self> {
        Rc::new(Self::new(client, env))
    }

    pub fn new(client: &Client, env: &LambdaEnv) -> Self {
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

impl RepositoryContainer for GeneralRepositories {
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

impl AllRepos for GeneralRepositories {}
