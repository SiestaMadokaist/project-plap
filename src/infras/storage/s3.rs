use std::{
    cell::OnceCell,
    path::{Path, PathBuf},
    process::Stdio,
};

use aws_config::SdkConfig;
use aws_sdk_s3::{primitives::ByteStream, types::ObjectCannedAcl::PublicRead, Client};
use tokio::process::Command;

use crate::{
    application::ports::clients::storage::StorageClient,
    domain::{
        commands::compute::ComputeRegion,
        errors::DomainError,
        storage::{StorageBucket, StoragePath, StoragePrefix},
    },
};

pub struct S3Storage {
    client: Client,
    region: ComputeRegion,
    bucket: String,
    bucket_region: OnceCell<String>,
    /*
     * @desecription
     * valid prefix = ""
     * this will generate:
     * "" + <remote_path> => "s3://<bucket>/remote_path/to/target/file";
     * remote_path must not start with "/", so the "/", if needed should be added on the prefix
     *
     * another valid prefix = "root/"
     * this will generate:
     * "root/" + <remote_path> => "s3://<bucket>/root/remote_path/to/target/file"
     *
     * prefix = "/"
     * this make ugly remote_path
     * "/" + <remote_path> => "s3://<bucket>//remote_path/to/target/file" <- double "//" after <bucket>
     */
    remote_prefix: String,
    local_workdir: String,
    max_size: i64, // in bytes (e.g: 50 GB) => (50 * 1024 * 1024 * 1024);
}

impl S3Storage {
    pub fn new(
        config: SdkConfig,
        region: ComputeRegion,
        bucket: String,
        remote_prefix: String,
        local_workdir: String,
        max_size: i64,
    ) -> Self {
        let mut builder = config.to_builder();
        let sregion: String = region.into();
        builder.set_region(aws_config::Region::new(sregion));
        let reconfigured = builder.build();
        let client = aws_sdk_s3::Client::new(&reconfigured);
        Self {
            client,
            region,
            bucket,
            bucket_region: OnceCell::new(),
            remote_prefix,
            local_workdir,
            max_size,
        }
    }

    async fn bucket_region(&self) -> Result<ComputeRegion, DomainError> {
        if let Some(region) = self.bucket_region.get() {
            return ComputeRegion::try_from(region.as_str())
                .map_err(|e| DomainError::Prerequisite(e.to_string()));
        }

        let out = self
            .client
            .get_bucket_location()
            .bucket(&self.bucket)
            .send()
            .await
            .map_err(|e| DomainError::Disconnected(e.to_string()))?;

        // an empty/absent constraint means the bucket lives in us-east-1
        let region = out
            .location_constraint()
            .map(|c| c.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("us-east-1")
            .to_string();

        let computed = ComputeRegion::try_from(region.as_str())
            .map_err(|e| DomainError::Prerequisite(e.to_string()))?;

        // best-effort cache; if another call already won the race, its value is
        // equally valid so there's nothing to reconcile.
        let _ = self.bucket_region.set(region);

        Ok(computed)
    }

    async fn is_same_region(&self) -> Result<bool, DomainError> {
        let bucket_region = self.bucket_region().await?;
        Ok(bucket_region == self.region)
    }

    fn key(&self, path: &StoragePath) -> String {
        format!("{}{}", self.remote_prefix, path.0)
    }

    fn assert_within_limit(&self, size: i64) -> Result<(), DomainError> {
        if size > self.max_size {
            return Err(DomainError::TransferTooLarge {
                size: size as u64,
                limit: self.max_size as u64,
            });
        }
        Ok(())
    }

    // sums the size of every object under `key` and reports whether it's a
    // single exact-match object (plain `cp`) or a prefix (`cp --recursive`).
    async fn remote_extent(&self, key: &str) -> Result<(i64, bool), DomainError> {
        let mut total: i64 = 0;
        let mut count: u32 = 0;
        let mut exact_match = false;
        let mut token: Option<String> = None;

        loop {
            let mut req = self
                .client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(key);
            if let Some(t) = &token {
                req = req.continuation_token(t);
            }
            let out = req
                .send()
                .await
                .map_err(|e| DomainError::Disconnected(e.to_string()))?;

            for obj in out.contents() {
                count += 1;
                total += obj.size().unwrap_or(0);
                if obj.key() == Some(key) {
                    exact_match = true;
                }
            }

            token = match out.is_truncated() {
                Some(true) => out.next_continuation_token().map(str::to_string),
                _ => None,
            };
            if token.is_none() {
                break;
            }
        }

        if count == 0 {
            return Err(DomainError::Prerequisite(format!(
                "no objects found under s3://{}/{key}",
                self.bucket
            )));
        }

        Ok((total, !(count == 1 && exact_match)))
    }

    fn local_size(path: &Path) -> std::io::Result<i64> {
        let meta = std::fs::metadata(path)?;
        if meta.is_dir() {
            let mut total: i64 = 0;
            for entry in std::fs::read_dir(path)? {
                total += Self::local_size(&entry?.path())?;
            }
            Ok(total)
        } else {
            Ok(meta.len() as i64)
        }
    }

    async fn spawn_cp(&self, args: &Vec<String>) -> Result<(), DomainError> {
        let mut cmd = Command::new("aws");
        cmd.args(args);
        tracing::info!("spawning aws command: aws {}", args.join(" "));
        let output = cmd
            .stdin(Stdio::null())
            .output()
            .await
            .map_err(|e| DomainError::Disconnected(e.to_string()))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DomainError::Disconnected(format!(
                "aws s3 cp failed: {stderr}"
            )));
        }
        Ok(())
    }
}

impl S3Storage {
    fn download_args(
        &self,
        remote: &StoragePath,
        local: &PathBuf,
        recursive: bool,
    ) -> Result<Vec<String>, DomainError> {
        let key = self.key(remote);
        let src = format!("s3://{}/{key}", self.bucket);
        let dst = local
            .to_str()
            .ok_or_else(|| DomainError::Serialize("local path is not valid UTF-8".into()))?;
        let mut args = vec!["s3".into(), "cp".into(), src.into(), dst.into()];
        if recursive {
            args.push("--recursive".into());
        }
        Ok(args)
    }

    fn upload_args(
        &self,
        remote: &StoragePath,
        local: &PathBuf,
        recursive: bool,
    ) -> Result<Vec<String>, DomainError> {
        let key = self.key(remote);
        let src = local
            .to_str()
            .ok_or_else(|| DomainError::Serialize("local path is not valid UTF-8".into()))?;
        let dst = format!("s3://{}/{key}", self.bucket);
        let mut args = vec!["s3".into(), "cp".into(), src.into(), dst.into()];
        if recursive {
            args.push("--recursive".into());
        }
        Ok(args)
    }
}

impl StorageClient for S3Storage {
    fn provider_name() -> String {
        "s3".into()
    }

    #[cfg(feature = "datatransfer")]
    fn abs_path(&self, path: &PathBuf) -> PathBuf {
        let path_str = path.to_str().unwrap_or_default();
        let dst = format!("{}{}", self.local_workdir, path_str);
        PathBuf::from(dst)
    }

    fn bucket(&self) -> StorageBucket {
        StorageBucket(self.bucket.clone())
    }

    #[cfg(feature = "datatransfer")]
    async fn download(&self, remote: &StoragePath, local: &PathBuf) -> Result<(), DomainError> {
        let same_region = self.is_same_region().await?;
        if !same_region {
            let err = DomainError::BillOptimization(
                "download capability is only allowed for same region".into(),
            );
            return Err(err);
        }
        let key = self.key(remote);
        let (size, recursive) = self.remote_extent(&key).await?;
        self.assert_within_limit(size)?;
        let args = self.download_args(remote, local, recursive)?;
        self.spawn_cp(&args).await
    }

    #[cfg(feature = "datatransfer")]
    async fn upload(&self, local: &PathBuf, remote: &StoragePath) -> Result<(), DomainError> {
        let same_region = self.is_same_region().await?;
        if !same_region {
            let err = DomainError::BillOptimization(
                "upload capability is only allowed for same region".into(),
            );
            return Err(err);
        }
        let size = Self::local_size(local).map_err(|e| DomainError::Disconnected(e.to_string()))?;
        self.assert_within_limit(size)?;
        let recursive = local.is_dir();
        let args = self.upload_args(remote, local, recursive)?;
        self.spawn_cp(&args).await
        // self.spawn_cp(src, &dst, recursive).await
    }

    async fn read(&self, path: &StoragePath) -> Result<String, DomainError> {
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

    fn public_url(&self, path: &StoragePath) -> String {
        return format!(
            "https://{}.s3.{}.amazonaws.com/{}",
            self.bucket, self.region, path.0,
        );
    }

    async fn ls(&self, prefix: &StoragePrefix) -> Vec<String> {
        let result = self
            .client
            .list_objects()
            .bucket(&self.bucket)
            .prefix(prefix.0.clone())
            .send()
            .await;
        let vs = match result {
            Err(_) => vec![],
            Ok(items) => {
                let vecs = items.contents.unwrap_or(vec![]);
                let vocs = vecs
                    .iter()
                    .map(|o| o.clone().key.unwrap_or(String::from("")))
                    .filter(|x| x.len() > 0)
                    .map(|x| String::from(x));
                let vcs: Vec<String> = vocs.collect();
                vcs
            }
        };
        vs
    }

    #[cfg(feature = "future")]
    async fn versions(&self, path: &StoragePath) -> Result<ItemVersion, DomainError> {
        todo!();
    }

    async fn write(&self, path: &StoragePath, data: &Vec<u8>) -> Result<(), DomainError> {
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
        tracing::debug!("file written into {path}");
        Ok(())
    }
}

#[cfg(feature = "future")]
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    #[test]
    fn test_download_args() -> Result<(), DomainError> {
        let config = SdkConfig::builder()
            .behavior_version(aws_config::BehaviorVersion::latest())
            .build();
        let storage = S3Storage::new(
            config,
            ComputeRegion::AWSUsEast1,
            "test-bucket".into(),
            "models/".into(),
            "models/".into(),
            100,
        );
        let remote = StoragePath("checkpoints/1322/x.safetensors".into());
        let local = PathBuf::from_str("models/checkpoints/x.safetensors")
            .map_err(|x| DomainError::UnknownError(x.to_string()))?;
        let recursive = true;
        let args = storage.download_args(&remote, &local, recursive)?;
        let expected =
            "s3 cp s3://test-bucket/models/checkpoints/1322/x.safetensors models/checkpoints/x.safetensors --recursive";
        let actual = args.join(" ");
        assert_eq!(&actual, expected);
        Ok(())
    }

    #[test]
    fn test_upload_args() -> Result<(), DomainError> {
        let config = SdkConfig::builder()
            .behavior_version(aws_config::BehaviorVersion::latest())
            .build();
        let storage = S3Storage::new(
            config,
            ComputeRegion::AWSUsEast1,
            "test-bucket".into(),
            "models/".into(),
            "models/".into(),
            100,
        );
        let remote = StoragePath("checkpoints/1322/x.safetensors".into());
        let local = PathBuf::from_str("models/checkpoints/x.safetensors")
            .map_err(|x| DomainError::UnknownError(x.to_string()))?;
        let recursive = true;
        let args = storage.upload_args(&remote, &local, recursive)?;
        let expected =
            "s3 cp models/checkpoints/x.safetensors s3://test-bucket/models/checkpoints/1322/x.safetensors --recursive";
        let actual = args.join(" ");
        assert_eq!(&actual, expected);
        Ok(())
    }
}
