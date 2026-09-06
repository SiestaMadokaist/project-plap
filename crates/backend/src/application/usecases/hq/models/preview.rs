use std::time::Duration;

use crate::application::ports::{
    clients::{container::HasModelStorage, storage::StorageClient},
    usecase::UsecaseAPI,
};
use domain::errors::DomainError;
use dto::resources::models as resource;
use pkg::macros::trait_clients;

/// Presigned GET urls / inline text last for 5 minutes — long enough to open the
/// modal, short enough not to leak.
const PREVIEW_TTL: Duration = Duration::from_secs(300);

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
        for path in &self.payload.images {
            image_urls.push(storage.presigned_get(path, PREVIEW_TTL).await?);
        }
        let json = match &self.payload.json {
            Some(path) => Some(storage.read(path).await?),
            None => None,
        };

        Ok(resource::PreviewResponse { image_urls, json })
    }
}
