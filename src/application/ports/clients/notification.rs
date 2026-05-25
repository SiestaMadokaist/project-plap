use crate::domain::errors::DomainError;

#[allow(async_fn_in_trait)]
pub trait NotificationClient {
    async fn notify(&self, info: &str) -> Result<(), DomainError>;
}
