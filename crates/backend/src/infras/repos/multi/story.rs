use domain::{
    errors::DomainError,
    storage::{StoragePath, StoragePrefix},
    storyline::{StoryTemplateId, Storyline},
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

impl S3StoryRepository {
    pub fn new(storage: S3Storage) -> Self {
        Self { storage }
    }

    /// stories are scoped to their owner: the object key (below the storage's
    /// `remote_prefix`) is always `<username>/<story_id>`.
    fn key(owner: &Username, id: &StoryTemplateId) -> StoragePath {
        StoragePath(format!("{}/{}", owner.0, id.0))
    }
}

impl StoryTemplateRepository for S3StoryRepository {
    async fn get(&self, owner: &Username, id: &StoryTemplateId) -> Result<Storyline, DomainError> {
        let path = Self::key(owner, id);
        let data = self.storage.read(&path).await?;
        let json = serde_json::from_str::<Storyline>(&data)?;
        Ok(json)
    }

    async fn list(&self, owner: &Username) -> Result<Vec<StoryTemplateId>, DomainError> {
        let prefix = StoragePrefix(owner.0.clone());
        let items = self.storage.ls(&prefix).await?;
        // `ls` yields full object keys (`<remote_prefix>/<username>/<story_id>`);
        // the story id is just the last path segment.
        let ids = items
            .into_iter()
            .filter_map(|p| p.0.rsplit('/').next().map(str::to_owned))
            .filter(|name| !name.is_empty())
            .map(StoryTemplateId)
            .collect();
        Ok(ids)
    }

    async fn write(&self, owner: &Username, payload: &Storyline) -> Result<Storyline, DomainError> {
        let path = Self::key(owner, payload.id());
        let data = serde_json::to_vec(payload)?;
        self.storage.write(&path, &data).await?;
        Ok(payload.clone())
    }

    async fn delete(&self, owner: &Username, id: &StoryTemplateId) -> Result<(), DomainError> {
        let path = Self::key(owner, id);
        self.storage.delete(&path).await
    }
}
