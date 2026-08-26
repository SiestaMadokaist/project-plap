use crate::{
    application::ports::{
        clients::{self, storage::StorageClient},
        repository::{self},
    },
    domain::{errors::DomainError, storage::StoragePath},
    pkg::{
        exif::{comfyui::ComfyUI, traits::Exif},
        macros::{trait_clients, trait_repos},
        types::time::Timestamp,
    },
};
use std::{
    path::{Path, PathBuf},
    rc::Rc,
};
use tokio::sync::OnceCell;

struct Memo {
    data: OnceCell<Vec<u8>>,
}

impl Memo {
    fn new() -> Self {
        Self {
            data: OnceCell::new(),
        }
    }
}

trait_clients!(
    SaveOutputClient,
    clients::container::HasOutputStorage,
    clients::container::HasNotification
);
trait_repos!(
    SaveOutputRepos,
    repository::container::HasHotReload,
    repository::container::HasPromptHistory
);
pub struct SaveOutput<C: SaveOutputClient, R: SaveOutputRepos> {
    clients: Rc<C>,
    repos: Rc<R>,
    workdir: PathBuf,
    path: PathBuf,
    now: Timestamp,
    memo: Memo,
}

impl<C: SaveOutputClient, R: SaveOutputRepos> SaveOutput<C, R> {
    pub fn new(
        clients: Rc<C>,
        repos: Rc<R>,
        workdir: PathBuf,
        path: PathBuf,
        now: Timestamp,
    ) -> Self {
        Self {
            clients,
            repos,
            workdir,
            path,
            now,
            memo: Memo::new(),
        }
    }

    async fn read_exif(&self) -> anyhow::Result<Exif<ComfyUI>> {
        let data = self.ioread().await?;
        let exif = Exif::<ComfyUI>::new(data.clone());
        Ok(exif)
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

    async fn ioread(&self) -> anyhow::Result<&Vec<u8>> {
        let result = self
            .memo
            .data
            .get_or_try_init(async || tokio::fs::read(&self.path).await)
            .await?;
        Ok(result)
    }

    fn relative_path(&self) -> PathBuf {
        self.path
            .strip_prefix(&self.workdir)
            .map(Path::to_path_buf)
            .unwrap_or_else(|_| self.path.clone())
    }

    async fn save_output(&self) -> anyhow::Result<()> {
        let c = self.clients.clone();
        let storage = c.output_storage();
        let data = self.ioread().await?;
        let remote_path = self.store_path();
        let local_path = self.relative_path();
        tracing::info!("uploading {} to {}", local_path.display(), storage.bucket());
        let err_msg = format!(
            "file transfer failure when uploading {} to {}",
            local_path.display(),
            storage.bucket()
        );
        storage
            .write(&remote_path, data)
            .await
            .map_err(|_| DomainError::ApiError(err_msg).into())
    }

    pub async fn exec(&self) -> anyhow::Result<()> {
        self.save_exif().await?;
        self.save_output().await?;
        Ok(())
    }
}
