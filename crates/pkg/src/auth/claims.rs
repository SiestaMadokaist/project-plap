use k256::ecdsa::SigningKey;
use serde::{Deserialize, Serialize};

use crate::{
    displayable,
    types::{strings::Hex, time::Timestamp},
};

/// what `Authorizer::authenticate` encodes into the JWT, and the minimum shape
/// `Authorizer::authorize`'s `D` needs to be able to deserialize.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Claims {
    /// 0x-prefixed, checksum-less hex Ethereum address recovered from the request's pubkey.
    pub sub: String,
    /// issued-at, unix seconds.
    pub iat: Timestamp,
    /// expiry, unix seconds.
    pub exp: Timestamp,
    /// server generated challenge.
    pub challenge: Hex,
    /// server signed challenge with server's own privkey.
    pub server_sign: Hex,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JWT(pub String);
displayable!(JWT);

impl JWT {
    pub fn new(claims: Claims) -> Self {
        let s = serde_json::to_string(&claims).expect("claims must alwayas be a valid json");
        JWT(s)
    }

    pub fn get(&self) -> Result<Claims, serde_json::Error> {
        let s = serde_json::from_str::<Claims>(&self.0);
        s
    }
}
