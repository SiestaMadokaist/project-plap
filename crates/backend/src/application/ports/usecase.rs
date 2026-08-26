use domain::errors::DomainError;
use dto::response::DTO;

#[allow(async_fn_in_trait)]
pub trait UsecaseAPI<C: DTO> {
    async fn exec(&self) -> Result<C, DomainError>;
}
