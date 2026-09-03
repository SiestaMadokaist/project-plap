use crate::application::ports::{
    clients::{self, storage::StorageClient},
    repository::{self},
};
use domain::{errors::DomainError, storage::StoragePath};
use pkg::{
    exif::{
        comfyui::ComfyUI,
        traits::{Exif, ExifTraits},
    },
    macros::{trait_clients, trait_repos},
    types::time::{Timestamp, TimestampMS},
};
use std::path::{Path, PathBuf};
use tokio::sync::OnceCell;

const FILE_EXT: &str = ".png";

struct Memo {
    data: OnceCell<Vec<u8>>,
    exif: OnceCell<Exif<ComfyUI>>,
}

impl Memo {
    fn new() -> Self {
        Self {
            data: OnceCell::new(),
            exif: OnceCell::new(),
        }
    }
}

trait_clients!(
    SaveOutputClient,
    clients::container::HasOutputStorage,
    clients::container::HasModelStorage,
    clients::container::HasNotification
);
trait_repos!(
    SaveOutputRepos,
    repository::container::HasHotReload,
    repository::container::HasPromptHistory
);
pub struct SaveOutput<'a, C: SaveOutputClient, R: SaveOutputRepos> {
    clients: &'a C,
    _repos: &'a R,
    workdir: PathBuf,
    path: PathBuf,
    now: TimestampMS,
    memo: Memo,
    // @todo: consider using hotreload instead.
    blacklist_tags: &'a Vec<String>,
}
impl<'a, C: SaveOutputClient, R: SaveOutputRepos> SaveOutput<'a, C, R> {
    pub fn new(
        clients: &'a C,
        repos: &'a R,
        workdir: PathBuf,
        path: PathBuf,
        now: TimestampMS,
        blacklist_tags: &'a Vec<String>,
    ) -> Self {
        Self {
            clients,
            _repos: repos,
            workdir,
            path,
            now,
            blacklist_tags,
            memo: Memo::new(),
        }
    }

    async fn read_exif(&self) -> Result<&Exif<ComfyUI>, DomainError> {
        let result: Result<&Exif<ComfyUI>, DomainError> = self
            .memo
            .exif
            .get_or_try_init(async || {
                let data = self
                    .ioread()
                    .await
                    .map_err(|x| DomainError::Prerequisite(x.to_string()))?;
                let e = Exif::<ComfyUI>::new(data.clone());
                Ok(e)
            })
            .await;
        result
    }

    /**
     * @todo!()
     * extract exif from image
     * store exif to bigquery
     */
    async fn save_exif(&self) -> anyhow::Result<()> {
        let _exif = self.read_exif().await?;
        Ok(())
    }

    fn store_path(&self) -> StoragePath {
        let now = self.now;
        let ds = now.to_datestring();
        let date_string = ds.unwrap_or("UNKNOWN-DATE".into());
        let filename = format!("{}.png", now.0);
        let s = format!("{}/{}", date_string, filename);
        StoragePath(s)
    }

    async fn ioread(&self) -> Result<&Vec<u8>, DomainError> {
        let result = self
            .memo
            .data
            .get_or_try_init(async || tokio::fs::read(&self.path).await)
            .await
            .map_err(|x| DomainError::UnknownError(x.to_string()))?;
        Ok(result)
    }

    fn relative_path(&self) -> PathBuf {
        self.path
            .strip_prefix(&self.workdir)
            .map(Path::to_path_buf)
            .unwrap_or_else(|_| self.path.clone())
    }

    async fn preview_path(&self) -> Result<StoragePath, DomainError> {
        let exif = self.read_exif().await?;
        let now = Timestamp::now();
        let sample_number = now.0 % 3;
        let model = exif
            .checkpoint()
            .map_err(|x| DomainError::Prerequisite(x.to_string()))?;
        // checkpoint is the loaded filename, e.g. `<name>.safetensors`; drop the
        // extension so the preview lands next to the model key as
        // `diffusion_models/<name>.image-<n>.png` (matched by the frontend).
        let stem = model.strip_suffix(".safetensors").unwrap_or(model);
        let path = format!(
            "comfyui/diffusion_models/{}.image-{}{}",
            stem, sample_number, FILE_EXT
        );
        Ok(StoragePath(path))
    }

    async fn save_preview(&self) -> Result<(), DomainError> {
        let exif = self.read_exif().await?;
        let blacklisted = self.blacklist_tags;
        let prompt = exif
            .positive()
            .map_err(|x| DomainError::Prerequisite(x.to_string()))?;

        // Preview must be SFW: if the positive prompt carries any blacklisted tag,
        // skip publishing the preview entirely (case-insensitive substring match;
        // empty entries are ignored so they can't blacklist everything).
        let haystack = prompt.to_lowercase();
        if blacklisted.iter().any(|tag| {
            let tag = tag.trim().to_lowercase();
            !tag.is_empty() && haystack.contains(&tag)
        }) {
            tracing::info!("preview skipped: blacklisted tag in prompt");
            return Ok(());
        }

        let model_storage = self.clients.model_storage();
        let preview_path = self.preview_path().await?;
        let data = self.ioread().await?;
        model_storage.write(&preview_path, data).await?;
        Ok(())
    }

    async fn save_output(&self) -> Result<(), DomainError> {
        let c = self.clients;
        let output_storage = c.output_storage();
        let data = self.ioread().await?;
        let remote_path = self.store_path();
        let local_path = self.relative_path();
        tracing::info!(
            "uploading {} to {}",
            local_path.display(),
            output_storage.bucket()
        );
        let err_msg = format!(
            "file transfer failure when uploading {} to {}",
            local_path.display(),
            output_storage.bucket()
        );
        output_storage
            .write(&remote_path, data)
            .await
            .map_err(|_| DomainError::ApiError(err_msg))
    }

    pub async fn exec(&self) -> anyhow::Result<()> {
        self.save_exif().await?;
        self.save_output().await?;
        self.save_preview().await?;
        Ok(())
    }
}
