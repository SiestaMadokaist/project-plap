use crate::application::ports::{
    clients::{container::HasModelStorage, storage::StorageClient},
    usecase::UsecaseAPI,
};
use domain::errors::DomainError;
use dto::resources::models as resource;
use pkg::macros::trait_clients;
trait_clients!(IClients, HasModelStorage);

pub struct PreviewSvc<'a, C: IClients> {
    clients: &'a C,
    payload: resource::PreviewPayload,
}

impl<'a, C: IClients> PreviewSvc<'a, C> {
    pub fn new(clients: &'a C, payload: resource::PreviewPayload) -> Self {
        Self { clients, payload }
    }
}

impl<C: IClients> UsecaseAPI<resource::PreviewResponse> for PreviewSvc<'_, C> {
    async fn exec(&self) -> Result<resource::PreviewResponse, DomainError> {
        let storage = self.clients.model_storage();

        let mut image_urls = Vec::with_capacity(self.payload.images.len());
        for target in &self.payload.images {
            image_urls.push(
                storage
                    .presigned_get(&target.path, target.ttl.to_duration())
                    .await?,
            );
        }
        let json = match &self.payload.json {
            Some(target) => Some(storage.read(&target.path).await?),
            None => None,
        };
        let safetensors_url = match &self.payload.safetensors {
            Some(target) => Some(
                storage
                    .presigned_get(&target.path, target.ttl.to_duration())
                    .await?,
            ),
            None => None,
        };

        Ok(resource::PreviewResponse {
            image_urls,
            json,
            safetensors_url,
        })
    }
}
