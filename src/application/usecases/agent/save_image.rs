use crate::{
    application::ports::{
        clients::{self, storage::StorageClient},
        repository::{self},
    },
    domain::storage::StoragePath,
    pkg::{
        macros::{trait_clients, trait_repos},
        types::time::Timestamp,
    },
};
use std::{path::PathBuf, rc::Rc};
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
    SaveImageClient,
    clients::container::HasStorage,
    clients::container::HasNotification
);
trait_repos!(
    SaveImageRepos,
    repository::container::HasHotReload,
    repository::container::HasPromptHistory
);
pub struct SaveImage<C: SaveImageClient, R: SaveImageRepos> {
    clients: Rc<C>,
    repos: Rc<R>,
    path: PathBuf,
    now: Timestamp,
    memo: Memo,
}

impl<C: SaveImageClient, R: SaveImageRepos> SaveImage<C, R> {
    pub fn new(clients: Rc<C>, repos: Rc<R>, path: PathBuf, now: Timestamp) -> Self {
        Self {
            clients,
            repos,
            path,
            now,
            memo: Memo::new(),
        }
    }

    /**
     * @todo!()
     * extract exif from image
     * store exif to bigquery
     */
    async fn read_exif(&self) -> anyhow::Result<()> {
        let data = self.ioread().await;
        Ok(())
    }

    fn store_path(&self) -> StoragePath {
        let now = self.now;
        let ds = now.to_datestring();
        let date_string = ds.as_str();
        let path = self.path.to_str().unwrap_or("todo!(now)");
        let s = format!("{}/{}", date_string, path);
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

    async fn store_image(&self) -> anyhow::Result<()> {
        let c = self.clients.clone();
        let storage = c.storage();
        let data = self.ioread().await?;
        let path = self.store_path();
        storage.write(path, data).await.map_err(|_| todo!())
    }

    pub async fn exec(&self) -> anyhow::Result<()> {
        self.read_exif().await?;
        self.store_image().await?;
        Ok(())
    }
}
