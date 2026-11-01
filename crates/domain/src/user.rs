use pkg::{
    auth::claims::Username,
    types::{crypto::Eth0x, time::Timestamp},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: Username,
    /// `None` until the account is activated.
    pub activated_at: Option<Timestamp>,
    pub address: Eth0x,
    pub created_at: Timestamp,
    /// `None` until the first successful login.
    pub last_login: Option<Timestamp>,
}
