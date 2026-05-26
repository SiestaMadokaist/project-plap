use crate::{
    application::ports::repository::rc::{AllRepos, RepositoryContainer},
    infras::repos::dynamo::{translation::DDBTranslationRepository, user::DDBUserRepository},
};

pub struct DynamoRepositoryContainer {
    translation: DDBTranslationRepository,
    user: DDBUserRepository,
}

impl DynamoRepositoryContainer {
    pub fn new(tl: DDBTranslationRepository, u: DDBUserRepository) -> Self {
        Self {
            translation: tl,
            user: u,
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
