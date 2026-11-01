use serde::{Deserialize, Serialize};

use crate::{displayable, id_type, types::time::Timestamp};

id_type!(Username);

/// Payload of an issued session token. The HS256 MAC over these fields (keyed by the
/// server `secret`) is the only integrity check performed on it - there is no
/// separate asymmetric signature here, unlike the challenge exchange. Signing and
/// decoding live in the backend (`infras::authorizer::jwt`); this crate only carries the data.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthClaims {
    pub username: Username,
    /// 0x-prefixed, checksum-less hex Ethereum address the token was issued to.
    pub sub: String,
    /// issued-at, unix seconds.
    pub iat: Timestamp,
    /// expiry, unix seconds.
    pub exp: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JWT(pub String);
displayable!(JWT);
