use std::{path::PathBuf, rc::Rc};

use crate::{
    application::{
        ports::clients::storage::StorageClient,
        usecases::agent::traits::{AgentClients, AgentRepos},
    },
    domain::{errors::DomainError, storage::StoragePath},
};

pub struct SaveImage<C: AgentClients, R: AgentRepos> {
    clients: Rc<C>,
    repos: Rc<R>,
    path: PathBuf,
}

impl<C: AgentClients, R: AgentRepos> SaveImage<C, R> {
    pub fn new(clients: Rc<C>, repos: Rc<R>, path: PathBuf) -> Self {
        Self {
            clients,
            repos,
            path,
        }
    }

    /**
     * extract exif from image
     * store exif to bigquery
     */
    async fn store_exif(&self) -> anyhow::Result<()> {
        todo!();
    }

    async fn ioreader(&self) -> anyhow::Result<Vec<u8>> {
        let bytes = tokio::fs::read(&self.path).await?;
        Ok(bytes)
    }

    async fn store_image(&self) -> anyhow::Result<()> {
        let c = self.clients.clone();
        let storage = c.storage();
        let data = self.ioreader().await?;
        let path = self.path.to_str().map(String::from).unwrap_or_default();
        if path == "" {
            todo!(); // return Err;
        }
        storage.write(StoragePath(path), data).await?;
        todo!();
    }

    pub async fn exec(&self) -> anyhow::Result<()> {
        self.store_exif().await?;
        self.store_image().await?;
        Ok(())
    }
}
