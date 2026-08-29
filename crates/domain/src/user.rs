use pkg::{auth::ecdsa::AddressETH, macros::id_type, types::time::Timestamp};
use serde::{Deserialize, Serialize};

id_type!(UserId);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    /// `None` until the account is activated.
    pub activated_at: Option<Timestamp>,
    pub address: AddressETH,
    pub created_at: Timestamp,
    /// `None` until the first successful login.
    pub last_login: Option<Timestamp>,
}
