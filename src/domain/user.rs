use crate::pkg::{
    macros::id_type,
    types::{strings::Email, time::Timestamp},
};
use serde::{Deserialize, Serialize};

id_type!(UserId);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub email: Email,
    pub name: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
