use std::rc::Rc;

use crate::{
    application::ports::clients::{container::HasModelStorage, storage::StorageClient},
    domain::{commands::compute::ComputeRegion, errors::DomainError, storage::StoragePrefix},
    json_type,
};
use serde::{Deserialize, Serialize};
use serde_json;

pub struct GetListModel<C: HasModelStorage> {
    clients: Rc<C>,
    payload: Payload,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
    region: ComputeRegion,
}
json_type!(Payload);

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

    pub async fn exec(&self) -> Result<serde_json::Value, DomainError> {
        let result = self.run().await.map_err(|_| DomainError::Unhandled)?;
        Ok(result.into())
    }
}
