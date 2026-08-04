use aws_sdk_s3::{primitives::ByteStream, types::ObjectCannedAcl::PublicRead, Client};

use crate::{
    application::ports::clients::storage::StorageClient,
    domain::{
        errors::DomainError,
        storage::{StorageBucket, StoragePath},
    },
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

    async fn write(&self, path: StoragePath, data: Vec<u8>) -> Result<(), DomainError> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(self.key(&path))
            .body(ByteStream::from(data))
            .acl(PublicRead)
            .send()
            .await
            .map_err(|e| DomainError::Disconnected(e.to_string()))?;
        Ok(())
    }
}
