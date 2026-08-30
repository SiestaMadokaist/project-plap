use serde::{Deserialize, Serialize};

use crate::response::DTO;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ListMeta {
    page: u32,
    from: u32,
    to: u32,
    total: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListResponse<T> {
    data: Vec<T>,
    meta: ListMeta,
}

impl<T: DTO> ListResponse<T> {
    pub fn simple(data: Vec<T>) -> Self {
        Self::new(data, ListMeta::default())
    }

    pub fn new(data: Vec<T>, meta: ListMeta) -> Self {
        Self { data, meta }
    }
}

impl<T: DTO> DTO for ListResponse<T> {}
