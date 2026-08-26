#[cfg(feature = "datatransfer")]
use std::path::{Path, PathBuf};

use domain::{
    errors::DomainError,
    storage::{StorageBucket, StoragePath, StoragePrefix},
};

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait StorageClient {
    fn provider_name() -> String;
    fn bucket(&self) -> StorageBucket;
    async fn read(&self, path: &StoragePath) -> Result<String, DomainError>;
    async fn write(&self, path: &StoragePath, data: &[u8]) -> Result<(), DomainError>;
    fn public_url(&self, path: &StoragePath) -> String;
    async fn ls(&self, prefix: &StoragePrefix) -> Vec<String>;

    #[cfg(feature = "future")]
    async fn versions(&self, path: &StoragePath) -> Result<ItemVersion, DomainError>;

    #[cfg(feature = "datatransfer")]
    async fn upload(&self, local: &Path, remote: &StoragePath) -> Result<(), DomainError>;
    #[cfg(feature = "datatransfer")]
    async fn download(&self, remote: &StoragePath, local: &Path) -> Result<(), DomainError>;

    #[cfg(feature = "datatransfer")]
    fn abs_path(&self, path: &Path) -> PathBuf;
}
