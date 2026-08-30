use std::rc::Rc;

use crate::application::ports::{
    clients::{container::HasModelStorage, storage::StorageClient},
    usecase::UsecaseAPI,
};
use domain::{errors::DomainError, storage::StoragePath};
use dto::resources::{
    list::{ListMeta, ListResponse},
    models as resource,
};
use pkg::macros::trait_clients;

trait_clients!(IClients, HasModelStorage);
pub struct GetList<C: IClients> {
    clients: Rc<C>,
    payload: resource::GetListPayload,
    // auth: JWT,
}

impl<C: IClients> GetList<C> {
    pub fn new(clients: Rc<C>, payload: resource::GetListPayload) -> Self {
        Self {
            clients,
            // auth,
            payload,
        }
    }

    async fn run(&self) -> Result<ListResponse<StoragePath>, DomainError> {
        // let valid = self.auth
        let storage = self.clients.model_storage();
        let paths = storage.ls(&self.payload.prefix).await?;
        // let resp = resource::GetListResponse { paths };
        let resp = ListResponse::simple(paths);
        Ok(resp)
    }
}

impl<C: IClients> UsecaseAPI<ListResponse<StoragePath>> for GetList<C> {
    async fn exec(&self) -> Result<ListResponse<StoragePath>, DomainError> {
        let result = self.run().await?;
        Ok(result)
    }
}
