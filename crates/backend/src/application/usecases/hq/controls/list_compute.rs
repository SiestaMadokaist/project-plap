use domain::errors::DomainError;
use dto::resources::computes::{ComputeListPayload, ComputeListResponse};
use pkg::macros::trait_clients;

use crate::application::ports::{
    clients::{
        self,
        compute::{ComputeEngine, ComputeEngines},
    },
    usecase::UsecaseAPI,
};

trait_clients!(ListComputeClients, clients::container::HasEngines);

pub struct ListCompute<'a, C: ListComputeClients> {
    clients: &'a C,
    payload: ComputeListPayload,
}

impl<'a, C: ListComputeClients> ListCompute<'a, C> {
    pub fn new(clients: &'a C, payload: ComputeListPayload) -> Self {
        Self { clients, payload }
    }
}

impl<'a, C: ListComputeClients> UsecaseAPI<ComputeListResponse> for ListCompute<'a, C> {
    async fn exec(&self) -> Result<ComputeListResponse, DomainError> {
        let region = &self.payload.region;
        let engine = self
            .clients
            .engines()
            .get(region)
            .ok_or_else(|| DomainError::InvalidRegion(region.to_string()))?;
        let instances = engine.list().await?;
        Ok(ComputeListResponse { instances })
    }
}
