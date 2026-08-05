use crate::{
    application::{
        ports::clients::storage::StorageClient,
        usecases::agent::traits::{AgentClients, AgentRepos},
    },
    domain::storage::StoragePath,
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

pub struct SaveImage<C: AgentClients, R: AgentRepos> {
    clients: Rc<C>,
    repos: Rc<R>,
    path: PathBuf,
    memo: Memo,
}

impl<C: AgentClients, R: AgentRepos> SaveImage<C, R> {
    pub fn new(clients: Rc<C>, repos: Rc<R>, path: PathBuf) -> Self {
        Self {
            clients,
            repos,
            path,
            memo: Memo::new(),
        }
    }

    /**
     * extract exif from image
     * store exif to bigquery
     */
    async fn read_exif(&self) -> anyhow::Result<()> {
        let data = self.ioread().await;
        todo!();
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
        let path = self.path.to_str().map(String::from).unwrap_or_default();
        if path == "" {
            todo!(); // return Err;
        }
        storage
            .write(StoragePath(path), data)
            .await
            .map_err(|_| todo!())
    }

    pub async fn exec(&self) -> anyhow::Result<()> {
        self.read_exif().await?;
        self.store_image().await?;
        Ok(())
    }
}
