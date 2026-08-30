use domain::{
    storage::{StoragePath, StoragePrefix},
    storyline::Storyline,
};
use pkg::auth::claims::Username;

use crate::{
    application::ports::{
        clients::storage::StorageClient, repository::story::StoryTemplateRepository,
    },
    infras::storage::s3::S3Storage,
};

pub struct S3StoryRepository {
    storage: S3Storage,
}
impl StoryTemplateRepository for S3StoryRepository {
    async fn get(
        &self,
        id: &domain::storyline::StoryId,
    ) -> Result<domain::storyline::Storyline, domain::errors::DomainError> {
        let path = StoragePath(id.0.clone());
        let data = self.storage.read(&path).await?;
        let json = serde_json::from_str::<Storyline>(&data)?;
        Ok(json)
    }

    async fn list(
        &self,
        username: &Username,
    ) -> Result<Vec<domain::storyline::StoryId>, domain::errors::DomainError> {
        let prefix = StoragePrefix(username.0.clone());
        let items = self.storage.ls(&prefix).await?;
        Ok(vec![])
    }

    async fn write(
        &self,
        payload: &domain::storyline::Storyline,
    ) -> Result<domain::storyline::Storyline, domain::errors::DomainError> {
        todo!()
    }
}
