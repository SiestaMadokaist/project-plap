use std::rc::Rc;

use aws_config::SdkConfig;
use backend::{
    application::ports::repository::container::{
        HasAgentCommand, HasHotReload, HasStoryTemplate, HasUser,
    },
    constant::ddb::DDBTable,
    infras::{
        repos::{
            dynamo::{
                agent_command::DDBAgentCommandRepository, hotreload::DDBHotReloadRepository,
                user::DDBUserRepository,
            },
            multi::story::S3StoryRepository,
        },
        storage::s3::S3Storage,
    },
};

use crate::env::ApiEnv;

pub struct ApiRepos {
    agent_command: DDBAgentCommandRepository,
    hotreload: DDBHotReloadRepository,
    user: DDBUserRepository,
    story: S3StoryRepository,
}

impl ApiRepos {
    pub fn rc(env: &ApiEnv, config: &SdkConfig) -> Rc<Self> {
        Rc::new(Self::new(env, config))
    }

    pub fn new(env: &ApiEnv, config: &SdkConfig) -> Self {
        let stage = env.stage();
        let client = aws_sdk_dynamodb::Client::new(config);
        let storage = S3Storage::new(
            config.clone(),
            env.template_region,
            env.template_bucket.clone(),
            env.template_prefix.clone(),
            "tmp/".into(),
            10 * 1024 * 1024,
            None,
        );
        Self {
            agent_command: DDBAgentCommandRepository::new(
                client.clone(),
                DDBTable::AgentCommands.table_name(stage),
            ),
            hotreload: DDBHotReloadRepository::new(
                client.clone(),
                DDBTable::HotReloads.table_name(stage),
            ),
            user: DDBUserRepository::new(client.clone(), DDBTable::Users.table_name(stage)),
            story: S3StoryRepository::new(storage),
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

impl HasUser for ApiRepos {
    type User = DDBUserRepository;
    fn user(&self) -> &Self::User {
        &self.user
    }
}

impl HasStoryTemplate for ApiRepos {
    type StoryTemplate = S3StoryRepository;
    fn story_template(&self) -> &Self::StoryTemplate {
        &self.story
    }
}
