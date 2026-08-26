use serde::{Deserialize, Serialize};

/// what `Authorizer::authenticate` encodes into the JWT, and the minimum shape
/// `Authorizer::authorize`'s `D` needs to be able to deserialize.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Claims {
    /// 0x-prefixed, checksum-less hex Ethereum address recovered from the request's pubkey.
    pub sub: String,
    /// issued-at, unix seconds.
    pub iat: i64,
    /// expiry, unix seconds.
    pub exp: i64,
}
