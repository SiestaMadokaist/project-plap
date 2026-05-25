use crate::{
    application::dto::base::DTO,
    domain::{errors::DomainError, translation::TranslationProgress},
};

/** @todo */
pub type TranslationResponse = TranslationProgress;
pub struct TranslationDTO {
    translation: TranslationResponse,
}

impl TranslationDTO {
    pub fn new(tl: TranslationProgress) -> Self {
        TranslationDTO { translation: tl }
    }
}

impl DTO<TranslationProgress> for TranslationDTO {
    fn represent(self) -> Result<TranslationResponse, DomainError> {
        return Ok(self.translation);
    }
}
