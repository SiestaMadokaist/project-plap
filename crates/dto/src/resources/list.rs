use serde::{Deserialize, Serialize};

use crate::response::DTO;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ListMeta {
    pub page: u32,
    pub from: u32,
    pub to: u32,
    pub total: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListResponse<T> {
    pub list: Vec<T>,
    #[serde(flatten)]
    pub meta: ListMeta,
}

impl<T: DTO> ListResponse<T> {
    pub fn simple(data: Vec<T>) -> Self {
        Self::new(data, ListMeta::default())
    }

    pub fn new(list: Vec<T>, meta: ListMeta) -> Self {
        Self { list, meta }
    }
}
