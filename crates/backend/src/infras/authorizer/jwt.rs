use jsonwebtoken::{
    decode, encode, errors::ErrorKind, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use pkg::auth::{
    claims::{AuthClaims, JWT},
    errors::AuthError,
};

/// Encode and HS256-sign `claims` with `secret`.
pub fn sign(claims: &AuthClaims, secret: &[u8]) -> Result<JWT, AuthError> {
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret),
    )
    .map(JWT)
    .map_err(|e| AuthError::TokenIssue(e.to_string()))
}

/// Verify the HS256 signature and `exp` with `secret`, returning the claims.
pub fn decode_claims(jwt: &JWT, secret: &[u8]) -> Result<AuthClaims, AuthError> {
    let mut validation = Validation::new(Algorithm::HS256);
    // no `aud` claim is issued, so don't require/validate one
    validation.validate_aud = false;
    decode::<AuthClaims>(&jwt.0, &DecodingKey::from_secret(secret), &validation)
        .map(|data| data.claims)
        .map_err(|e| match e.kind() {
            ErrorKind::ExpiredSignature => AuthError::Expired,
            _ => AuthError::TokenIssue(e.to_string()),
        })
}
