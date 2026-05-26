use crate::{
    application::dto::base::DTO,
    domain::{errors::DomainError, translation::TranslationDomain},
};

/** @todo */
pub type TranslationResponse = TranslationDomain;
pub struct TranslationDTO {
    translation: TranslationResponse,
}

impl TranslationDTO {
    pub fn new(tl: TranslationDomain) -> Self {
        TranslationDTO { translation: tl }
    }
}

impl DTO<TranslationDomain> for TranslationDTO {
    fn represent(self) -> Result<TranslationResponse, DomainError> {
        return Ok(self.translation);
    }
}
