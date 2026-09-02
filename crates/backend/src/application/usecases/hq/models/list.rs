use crate::application::ports::{
    clients::{container::HasModelStorage, storage::StorageClient},
    usecase::UsecaseAPI,
};
use domain::{errors::DomainError, storage::StoragePath};
use dto::resources::{list::ListResponse, models as resource};
use pkg::macros::trait_clients;

trait_clients!(IClients, HasModelStorage);
pub struct GetListSvc<'a, C: IClients> {
    clients: &'a C,
    payload: resource::GetListPayload,
    // auth: JWT,
}

impl<'a, C: IClients> GetListSvc<'a, C> {
    pub fn new(clients: &'a C, payload: resource::GetListPayload) -> Self {
        Self {
            clients,
            // auth,
            payload,
        }
    }

    async fn run(&self) -> Result<resource::GetListResponse, DomainError> {
        // let valid = self.auth
        let storage = self.clients.model_storage();
        let paths = storage.ls(&self.payload.prefix).await?;
        let resp = ListResponse::simple(paths);
        Ok(resp)
    }
}

impl<C: IClients> UsecaseAPI<resource::GetListResponse> for GetListSvc<'_, C> {
    async fn exec(&self) -> Result<ListResponse<StoragePath>, DomainError> {
        let result = self.run().await?;
        Ok(result)
    }
}
