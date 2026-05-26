use crate::domain::user::UserId;

pub fn translation_table_name() -> String {
    std::env::var("TRANSLATION_TABLE").unwrap_or_else(|_| "production-translations".to_string())
}

pub fn user_table_name() -> String {
    std::env::var("USER_TABLE").unwrap_or_else(|_| "production-users".to_string())
}

// Single-table design key schema: USER#{id} / METADATA
pub fn user_pk(id: &UserId) -> String {
    format!("USER#{}", id.0)
}

pub const USER_SK: &str = "METADATA";
