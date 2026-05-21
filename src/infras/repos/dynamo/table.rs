use crate::domain::user::UserId;

pub const TABLE_NAME_ENV: &str = "DYNAMO_TABLE";
pub const DEFAULT_TABLE_NAME: &str = "rust-api";

pub fn table_name() -> String {
    std::env::var(TABLE_NAME_ENV).unwrap_or_else(|_| DEFAULT_TABLE_NAME.to_string())
}

// Single-table design key schema: USER#{id} / METADATA
pub fn user_pk(id: &UserId) -> String {
    format!("USER#{}", id.0)
}

pub const USER_SK: &str = "METADATA";
