use crate::domain::{
    errors::DomainError,
    storage::{StorageBucket, StoragePath},
};

#[allow(async_fn_in_trait)]
pub trait StorageClient {
    fn provider_name() -> String;
    fn bucket(&self) -> StorageBucket;
    async fn read(&self, path: StoragePath) -> Result<String, DomainError>;
    async fn write(&self, path: StoragePath, data: Vec<u8>) -> Result<(), DomainError>;
    fn public_url(&self, path: StoragePath) -> String;
}
