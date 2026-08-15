use std::rc::Rc;

use crate::{
    application::ports::clients::{container::HasModelStorage, storage::StorageClient},
    domain::{errors::DomainError, storage::StoragePrefix},
    json_type,
    pkg::macros::trait_clients,
};
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
    prefix: StoragePrefix,
}
json_type!(Payload);

trait_clients!(IClients, HasModelStorage);
pub struct GetListModel<C: IClients> {
    clients: Rc<C>,
    payload: Payload,
}

impl<C: IClients> GetListModel<C> {
    pub fn new(clients: Rc<C>, payload: Payload) -> Self {
        Self { clients, payload }
    }

    async fn run(&self) -> anyhow::Result<Vec<String>> {
        let storage = self.clients.model_storage();
        let items = storage.ls(&self.payload.prefix).await;
        Ok(items)
    }

    pub async fn exec(&self) -> Result<serde_json::Value, DomainError> {
        let result = self.run().await.map_err(|_| DomainError::Unhandled)?;
        Ok(result.into())
    }
}
