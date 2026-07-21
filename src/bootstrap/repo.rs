use aws_sdk_dynamodb::Client;

use crate::{
    application::ports::repository::rc::{AllRepos, RepositoryContainer},
    config::env::Env,
    infras::repos::dynamo::{translation::DDBTranslationRepository, user::DDBUserRepository},
};

pub struct DynamoRepositoryContainer {
    translation: DDBTranslationRepository,
    user: DDBUserRepository,
}

impl DynamoRepositoryContainer {
    pub fn new(client: &Client, env: &Env) -> Self {
        Self {
            translation: DDBTranslationRepository::new(
                client.clone(),
                env.translation_table.clone(),
            ),
            user: DDBUserRepository::new(client.clone(), env.user_table.clone()),
        }
    }
}

impl RepositoryContainer for DynamoRepositoryContainer {
    type Translation = DDBTranslationRepository;
    type User = DDBUserRepository;
    fn translation(&self) -> &Self::Translation {
        return &self.translation;
    }

    fn user(&self) -> &Self::User {
        return &self.user;
    }
}

impl AllRepos for DynamoRepositoryContainer {}
