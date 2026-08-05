use aws_sdk_s3::{
    primitives::ByteStream,
    types::{ObjectCannedAcl::PublicRead, ObjectVersion},
    Client,
};
use chrono::DateTime;

use crate::{
    application::ports::clients::storage::StorageClient,
    domain::{
        errors::DomainError,
        storage::{ItemVersion, StorageBucket, StoragePath},
    },
    pkg::types::time::TimestampMS,
};

pub struct S3Storage {
    client: Client,
    bucket: String,
    prefix: String,
}

impl S3Storage {
    pub fn new(client: Client, bucket: String, prefix: String) -> Self {
        Self {
            client,
            bucket,
            prefix,
        }
    }

    fn key(&self, path: &StoragePath) -> String {
        format!("{}{}", self.prefix, path.0)
    }
}

impl StorageClient for S3Storage {
    fn provider_name() -> String {
        "s3".into()
    }

    fn bucket(&self) -> StorageBucket {
        StorageBucket(self.bucket.clone())
    }

    async fn read(&self, path: StoragePath) -> Result<String, DomainError> {
        let out = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(self.key(&path))
            .send()
            .await
            .map_err(|e| DomainError::Disconnected(e.to_string()))?;

        let bytes = out
            .body
            .collect()
            .await
            .map_err(|e| DomainError::Disconnected(e.to_string()))?
            .into_bytes();

        String::from_utf8(bytes.to_vec()).map_err(|e| DomainError::Serialize(e.to_string()))
    }

    fn public_url(&self, path: StoragePath) -> String {
        return format!(
            "https://{}.s3.ap-southeast-1.amazonaws.com/{}",
            self.bucket, path.0,
        );
    }

    async fn ls(&self, prefix: crate::domain::storage::StoragePrefix) -> Vec<String> {
        todo!();
    }

    async fn versions(&self, path: StoragePath) -> Result<ItemVersion, DomainError> {
        todo!();
    }

    async fn write(&self, path: StoragePath, data: &Vec<u8>) -> Result<(), DomainError> {
        let bytes = ByteStream::from(data.clone());
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(self.key(&path))
            .body(bytes)
            .acl(PublicRead)
            .send()
            .await
            .map_err(|e| DomainError::Disconnected(e.to_string()))?;
        Ok(())
    }
}

impl From<ObjectVersion> for ItemVersion {
    fn from(value: ObjectVersion) -> Self {
        ItemVersion {
            key: value.key,
            version_id: value.version_id,
            last_modified: value
                .last_modified
                .and_then(|x| DateTime::from_timestamp(x.secs(), 0)),
            size: value.size,
            e_tag: value.e_tag,
        }
    }
}
