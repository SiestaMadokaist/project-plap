use pkg::{
    auth::ecdsa::{AddressETH, Challenge, HexSign},
    types::time::{Second, Timestamp},
};
use serde::{Deserialize, Serialize};

use crate::response::DTO;

#[derive(Clone, Serialize, Deserialize)]
pub struct ReqChallenge {
    // to be validated, this address must exists in user database.
    address: AddressETH,
    // max one request/second per address
    iat: Timestamp,
    exp: Timestamp,
}
impl DTO for ReqChallenge {}

impl ReqChallenge {
    pub fn new(address: AddressETH, iat: Timestamp, ttl: Second) -> Self {
        Self {
            address,
            iat,
            exp: iat.add(ttl),
        }
    }

    pub fn address(&self) -> &AddressETH {
        &self.address
    }

    pub fn is_valid(
        &self,
        server_time: &Timestamp,
        tolerance: &Second,
        max_session_ttl: &Second,
    ) -> bool {
        // client time's drift must be within tolerance from server time
        let v0 = self.iat.sub(server_time).abs().lt(tolerance);
        // max exp = iat + MAX_SESSION_TTL
        let v1 = self.exp.sub(&self.iat).lt(max_session_ttl);
        v0 && v1
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ServerChallenge {
    address: AddressETH,
    iat: Timestamp,
    exp: Timestamp,
    // server generated random hex
    code: Challenge,
    // generated from server privkey.sign(code.0) or something.
    // sent as revalidator in client answer
    // server doesnt need state tracker like redis to track active challenge code.
    server_sign: HexSign,
}
impl DTO for ServerChallenge {}

impl ServerChallenge {
    fn to_sign(&self) -> String {
        format!(
            "{}-{}-{}-{}-{}",
            self.address.hex(),
            &self.iat.0,
            &self.exp.0,
            &self.code.hex(),
            &self.server_sign.hex(),
        )
    }

    /// EIP-191 personal-sign framing: what a wallet's personal_sign/eth_sign actually hashes
    /// and signs, not the raw `msg()`. Signing/verification must both go through this, or a
    /// correctly-produced signature will silently fail to verify.
    pub fn metamask_msg(&self) -> Challenge {
        let msg = self.to_sign();
        let s = format!("\x19Ethereum Signed Message:\n{}{}", msg.len(), msg);
        Challenge::new(s)
    }
}

pub struct ClientAnswer {
    pub challenge: ServerChallenge,
    pub client_sign: HexSign,
}
