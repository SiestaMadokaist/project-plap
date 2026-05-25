use crate::domain::errors::DomainError;

pub trait DTO<T> {
    fn represent(self) -> Result<T, DomainError>;
}
