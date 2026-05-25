use crate::{application::dto::base::DTO, domain::errors::DomainError};

#[allow(async_fn_in_trait)]
pub trait Usecase<R> {
    type Output: DTO<R>;
    async fn exec(self) -> Result<Self::Output, DomainError>;
}
