use dto::response::Placeholder;

use crate::application::ports::usecase::UsecaseAPI;

#[derive(Default, Debug)]
pub struct Healthcheck {}

impl UsecaseAPI<Placeholder> for Healthcheck {
    async fn exec(&self) -> Result<Placeholder, domain::errors::DomainError> {
        Ok(Placeholder(200))
    }
}
