use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::{
    application::ports::clients::{container::HasModelStorage, storage::StorageClient},
    domain::{commands::compute::ComputeRegion, storage::StoragePrefix},
};

pub struct GetListModel<C: HasModelStorage> {
    clients: Rc<C>,
    payload: Payload,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
    region: ComputeRegion,
}

impl<C: HasModelStorage> GetListModel<C> {
    pub fn new(clients: Rc<C>, payload: Payload) -> Self {
        Self { clients, payload }
    }

    async fn run(&self) -> anyhow::Result<Vec<String>> {
        let storage = self.clients.model_storage();
        let prefix = StoragePrefix("./models".into());
        let items = storage.ls(&prefix).await;
        Ok(items)
    }

    pub async fn exec(&self) -> anyhow::Result<serde_json::Value> {
        let result = self.run().await?;
        Ok(result.into())
    }
}
