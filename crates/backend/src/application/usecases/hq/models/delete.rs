use domain::errors::DomainError;
use dto::resources::models::{DeletePayload, DeleteResponse};
use pkg::trait_clients;

use crate::application::ports::{
    clients::{container::HasModelStorage, storage::StorageClient},
    usecase::UsecaseAPI,
};

trait_clients!(IClients, HasModelStorage);

pub struct DeleteSvc<'a, C: IClients> {
    clients: &'a C,
    payload: DeletePayload,
}

impl<'a, C: IClients> DeleteSvc<'a, C> {
    pub fn new(clients: &'a C, payload: DeletePayload) -> Self {
        Self {
            clients,
            // auth,
            payload,
        }
    }

    async fn run(&self) -> Result<DeleteResponse, DomainError> {
        let storage = self.clients.model_storage();
        for path in &self.payload.paths {
            storage.delete(path).await?;
        }
        Ok(DeleteResponse::default())
    }
}

impl<C: IClients> UsecaseAPI<DeleteResponse> for DeleteSvc<'_, C> {
    async fn exec(&self) -> Result<DeleteResponse, DomainError> {
        let result = self.run().await?;
        Ok(result)
    }
}
