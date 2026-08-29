use jsonwebtoken::{
    decode, encode, errors::ErrorKind, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};

use crate::{auth::errors::AuthError, displayable, types::time::Timestamp};

/// Payload of an issued session token. The HS256 MAC over these fields (keyed by the
/// server `secret`) is the only integrity check `JWT::decode` performs - there is no
/// separate asymmetric signature here, unlike the challenge exchange.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Claims {
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

impl JWT {
    /// Encode and HS256-sign `claims` with `secret`.
    pub fn sign(claims: &Claims, secret: &[u8]) -> Result<Self, AuthError> {
        encode(&Header::default(), claims, &EncodingKey::from_secret(secret))
            .map(JWT)
            .map_err(|e| AuthError::TokenIssue(e.to_string()))
    }

    /// Verify the HS256 signature and `exp` with `secret`, returning the claims.
    pub fn decode(&self, secret: &[u8]) -> Result<Claims, AuthError> {
        let mut validation = Validation::new(Algorithm::HS256);
        // no `aud` claim is issued, so don't require/validate one
        validation.validate_aud = false;
        decode::<Claims>(&self.0, &DecodingKey::from_secret(secret), &validation)
            .map(|data| data.claims)
            .map_err(|e| match e.kind() {
                ErrorKind::ExpiredSignature => AuthError::Expired,
                _ => AuthError::TokenIssue(e.to_string()),
            })
    }
}
