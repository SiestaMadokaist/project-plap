use serde::{Deserialize, Serialize};

use crate::{
    displayable,
    pkg::types::{
        strings::Hex,
        time::{Second, Timestamp},
    },
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthSecret(String);
displayable!(AuthSecret);

impl AuthSecret {
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

pub struct AuthReq {
    pub pubkey: Hex,
    pub signature: Hex,
    pub nonce: u32,
    pub client_time: Timestamp,
}

impl AuthReq {
    pub fn valid(&self, server_time: Timestamp, tolerance: Second) -> bool {
        let diff = server_time.sub(&self.client_time).abs();
        diff.lt(&tolerance)
    }

    pub fn msg(&self) -> String {
        format!("{}-{}", self.client_time.0, self.nonce)
    }
}

#[cfg(test)]
mod tests {
    use crate::pkg::{
        auth::authreq::AuthReq,
        types::{
            strings::Hex,
            time::{Second, Timestamp},
        },
    };

    #[test]
    fn valid_check() {
        let req = AuthReq {
            pubkey: Hex("".into()),
            signature: Hex("".into()),
            nonce: 1000,
            client_time: Timestamp(2000),
        };
        let valid = req.valid(Timestamp(2100), Second(200));
        assert!(valid)
    }

    #[test]
    fn time_travel_check() {
        let req = AuthReq {
            pubkey: Hex("".into()),
            signature: Hex("".into()),
            nonce: 1000,
            client_time: Timestamp(2000),
        };
        let valid = req.valid(Timestamp(1910), Second(200));
        assert!(valid)
    }

    #[test]
    fn past_check() {
        let req = AuthReq {
            pubkey: Hex("".into()),
            signature: Hex("".into()),
            nonce: 1000,
            client_time: Timestamp(2000),
        };
        let valid = req.valid(Timestamp(1700), Second(200));
        assert!(!valid)
    }

    #[test]
    fn invalid_check() {
        let req = AuthReq {
            pubkey: Hex("".into()),
            signature: Hex("".into()),
            nonce: 1000,
            client_time: Timestamp(2000),
        };
        let valid = req.valid(Timestamp(2201), Second(200));
        assert!(!valid)
    }
}
