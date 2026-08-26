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
}
displayable!(AuthError);
