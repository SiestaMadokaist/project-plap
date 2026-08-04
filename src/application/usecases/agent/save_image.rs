use std::{
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::application::usecases::agent::traits::{AgentClients, AgentRepos};

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

    async fn store_image(&self) -> anyhow::Result<()> {
        todo!();
    }

    pub async fn exec(&self) -> anyhow::Result<()> {
        self.store_exif().await?;
        self.store_image().await?;
        Ok(())
    }
}
