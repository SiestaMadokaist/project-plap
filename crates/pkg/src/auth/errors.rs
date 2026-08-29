use serde::{Deserialize, Serialize};

use crate::displayable;

#[derive(thiserror::Error, Serialize, Deserialize, Debug)]
pub enum AuthError {
    TODO,
    Deserialize,
    InvalidPubkey,
    InvalidSignature,
    VerificationFailed,
    TokenIssue(String),
    /// token is well-formed and untampered, but past its `exp`.
    Expired,
    /// token was issued before the server's `min_iat` cutoff - forced re-login.
    Revoked,
}
displayable!(AuthError);
