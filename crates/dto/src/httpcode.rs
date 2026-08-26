use domain::errors::DomainError;

pub(crate) fn code(e: &DomainError) -> u16 {
    500
}
