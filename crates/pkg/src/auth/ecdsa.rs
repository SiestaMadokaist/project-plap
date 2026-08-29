use k256::ecdsa::{Signature, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};

use crate::{auth::errors::AuthError, types::strings::Hex};

#[derive(Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct AddressETH(pub String);

impl AddressETH {
    pub fn hex(&self) -> Hex {
        let s = format!("0x{}", self.0);
        Hex(s)
    }
}
pub struct PubKey(VerifyingKey);
pub struct PrivKey(SigningKey);

#[derive(Serialize, Deserialize, Clone)]
pub struct Challenge(String);
impl Challenge {
    pub fn new(s: String) -> Self {
        Self(s)
    }
    pub fn hex(&self) -> Hex {
        todo!();
    }
}

impl PubKey {
    pub fn wrap(v: VerifyingKey) -> Self {
        todo!()
    }

    pub fn new(hex: Hex) -> Self {
        todo!()
    }

    pub fn verify(&self, msg: Challenge, sign: HexSign) -> Result<bool, AuthError> {
        todo!()
    }

    pub fn address(&self) -> Result<AddressETH, AuthError> {
        todo!()
    }
}

impl PrivKey {
    pub fn wrap(v: SigningKey) -> Self {
        todo!()
    }
    pub fn new(hex: Hex) -> Self {
        todo!()
    }
    pub fn hex(&self) -> Hex {
        todo!()
    }
    pub fn sign(&self, msg: Challenge) -> Signature {
        todo!()
    }
}

#[derive(Clone, Serialize, Debug, Deserialize)]
pub struct HexSign(Hex);

impl HexSign {
    pub fn wrap(v: Signature) -> Self {
        todo!()
    }
    pub fn new(hex: Hex) -> Self {
        todo!()
    }

    pub fn hex(&self) -> Hex {
        todo!()
    }

    pub fn get(&self) -> Signature {
        todo!()
    }

    pub fn pubkey(&self) -> PubKey {
        todo!()
    }
}
