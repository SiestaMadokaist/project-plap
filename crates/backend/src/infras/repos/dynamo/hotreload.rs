use aws_sdk_dynamodb::Client;

use crate::application::ports::repository::hot_reload::{HotReloadError, HotReloadRepository};
use domain::{hot_reload::DiffusionConfigDomain, user::UserId};
pub struct DDBHotReloadRepository {
    client: Client,
    table: String,
}

impl DDBHotReloadRepository {
    pub fn new(client: Client, table: String) -> Self {
        Self { client, table }
    }
}

impl HotReloadRepository for DDBHotReloadRepository {
    async fn diffusion_config(
        &self,
        _id: &UserId,
    ) -> Result<DiffusionConfigDomain, HotReloadError> {
        todo!()
    }
}
