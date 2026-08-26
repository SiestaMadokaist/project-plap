use crate::application::dto::base::DTO;
use domain::errors::DomainError;

pub struct VoidDTO {}

impl DTO<()> for VoidDTO {
    fn represent(self) -> Result<(), DomainError> {
        Ok(())
    }
}
