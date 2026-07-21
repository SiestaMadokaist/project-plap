use crate::domain::user::UserId;

// Single-table design key schema: USER#{id} / METADATA
pub fn user_pk(id: &UserId) -> String {
    format!("USER#{}", id.0)
}

pub const USER_SK: &str = "METADATA";
