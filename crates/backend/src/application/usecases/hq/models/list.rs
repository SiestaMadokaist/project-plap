use std::rc::Rc;

use crate::application::ports::{
    clients::{container::HasModelStorage, storage::StorageClient},
    usecase::UsecaseAPI,
};
use domain::errors::DomainError;
use dto::resources::models as resource;
use pkg::macros::trait_clients;
use serde_json;

trait_clients!(IClients, HasModelStorage);
pub struct GetList<C: IClients> {
    clients: Rc<C>,
    payload: resource::GetListPayload,
}

impl<C: IClients> GetList<C> {
    pub fn new(clients: Rc<C>, payload: resource::GetListPayload) -> Self {
        Self { clients, payload }
    }

    async fn run(&self) -> Result<resource::GetListResponse, DomainError> {
        let storage = self.clients.model_storage();
        let paths = storage.ls(&self.payload.prefix).await;
        let resp = resource::GetListResponse { paths };
        Ok(resp)
    }

    // pub async fn exec(&self) -> Result<serde_json::Value, DomainError> {
    // }
}

impl<C: IClients> UsecaseAPI<resource::GetListResponse> for GetList<C> {
    async fn exec(&self) -> Result<resource::GetListResponse, DomainError> {
        let result = self.run().await?;
        Ok(result)
    }
}
