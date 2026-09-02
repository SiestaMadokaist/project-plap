use crate::application::ports::{
    clients::{container::HasModelStorage, storage::StorageClient},
    usecase::UsecaseAPI,
};
use domain::errors::DomainError;
use dto::resources::models as resource;
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
        let payload = &self.payload;
        let storage = self.clients.model_storage();
        let tree = storage.ls(&payload.prefix, payload.recursive).await?;
        Ok(resource::GetListResponse {
            bucket: storage.bucket(),
            tree,
        })
    }
}

impl<C: IClients> UsecaseAPI<resource::GetListResponse> for GetListSvc<'_, C> {
    async fn exec(&self) -> Result<resource::GetListResponse, DomainError> {
        let result = self.run().await?;
        Ok(result)
    }
}
