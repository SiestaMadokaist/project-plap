use std::path::PathBuf;

use crate::domain::{
    errors::DomainError,
    storage::{ItemVersion, StorageBucket, StoragePath, StoragePrefix},
};

#[allow(async_fn_in_trait)]
pub trait StorageClient {
    fn provider_name() -> String;
    fn bucket(&self) -> StorageBucket;
    async fn read(&self, path: &StoragePath) -> Result<String, DomainError>;
    async fn write(&self, path: &StoragePath, data: &Vec<u8>) -> Result<(), DomainError>;
    fn public_url(&self, path: &StoragePath) -> String;
    async fn ls(&self, prefix: &StoragePrefix) -> Vec<String>;
    async fn versions(&self, path: &StoragePath) -> Result<ItemVersion, DomainError>;

    #[cfg(feature = "datatransfer")]
    async fn upload(&self, local: &PathBuf, remote: &StoragePath) -> Result<(), DomainError>;
    #[cfg(feature = "datatransfer")]
    async fn download(&self, remote: &StoragePath, local: &PathBuf) -> Result<(), DomainError>;
}
