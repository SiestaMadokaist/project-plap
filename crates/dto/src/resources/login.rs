use pkg::{
    auth::{
        claims::JWT,
        ecdsa::{AddressETH, Challenge, HexSign, PrivKey, PubKey},
    },
    json_type,
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
json_type!(ReqChallenge);

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

    pub fn iat(&self) -> Timestamp {
        self.iat
    }

    pub fn exp(&self) -> Timestamp {
        self.exp
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
    /// Mint a challenge for `address` and sign the canonical blob with the server key,
    /// so `login` can later confirm - without any stored state - that this server issued
    /// it and none of the fields were altered in transit.
    pub fn new(
        address: AddressETH,
        iat: Timestamp,
        exp: Timestamp,
        code: Challenge,
        signer: &PrivKey,
    ) -> Self {
        let server_sign = signer.sign(Challenge::new(Self::sign_blob(&address, iat, exp, &code)));
        Self {
            address,
            iat,
            exp,
            code,
            server_sign,
        }
    }

    pub fn address(&self) -> &AddressETH {
        &self.address
    }

    pub fn iat(&self) -> Timestamp {
        self.iat
    }

    pub fn exp(&self) -> Timestamp {
        self.exp
    }

    /// True iff `server_sign` is this server's signature over the current field values.
    pub fn verify_issued_by(&self, server_pub: &PubKey) -> bool {
        server_pub
            .verify(Challenge::new(self.to_sign()), self.server_sign.clone())
            .unwrap_or(false)
    }

    fn sign_blob(address: &AddressETH, iat: Timestamp, exp: Timestamp, code: &Challenge) -> String {
        format!("{}-{}-{}-{}", address.hex(), iat.0, exp.0, code.hex())
    }

    fn to_sign(&self) -> String {
        Self::sign_blob(&self.address, self.iat, self.exp, &self.code)
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

#[derive(Serialize, Deserialize, Clone)]
pub struct ClientAnswer {
    pub challenge: ServerChallenge,
    pub client_sign: HexSign,
}
impl DTO for ClientAnswer {}
json_type!(ClientAnswer);

/// The session token handed back on a successful login.
#[derive(Serialize, Deserialize, Clone)]
pub struct LoginResponse {
    pub token: JWT,
}
impl DTO for LoginResponse {}
json_type!(LoginResponse);

#[cfg(test)]
mod tests {
    use pkg::{
        auth::ecdsa::{recover, AddressETH, Challenge, HexSign, PrivKey},
        types::{
            strings::Hex,
            time::{Second, Timestamp},
        },
    };

    use super::{ReqChallenge, ServerChallenge};

    // standard iancoleman test bip44 from "test *11 + junk"
    struct IanColeman {
        #[allow(dead_code)]
        path: &'static str,
        address: Hex,
        #[allow(dead_code)]
        pubkey: Hex,
        privkey: Hex,
    }

    impl IanColeman {
        // m/44'/60'/0'/0/0
        fn account0() -> Self {
            Self {
                path: "m/44'/60'/0'/0/0",
                address: Hex("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".into()),
                pubkey: Hex(
                    "0x038318535b54105d4a7aae60c08fc45f9687181b4fdfc625bd1a753fa7397fed75".into(),
                ),
                privkey: Hex(
                    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".into(),
                ),
            }
        }

        // m/44'/60'/0'/0/1
        fn account1() -> Self {
            Self {
                path: "m/44'/60'/0'/0/1",
                address: Hex("0x70997970C51812dc3A010C7d01b50e0d17dc79C8".into()),
                pubkey: Hex(
                    "0x02ba5734d8f7091719471e7f7ed6b9df170dc70cc661ca05e688601ad984f068b0".into(),
                ),
                privkey: Hex(
                    "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d".into(),
                ),
            }
        }

        fn signer(&self) -> PrivKey {
            PrivKey::new(self.privkey.clone()).expect("fixture privkey is a valid scalar")
        }

        /// what `PubKey::address` produces: lowercase, no `0x`, no EIP-55 checksum.
        fn address_norm(&self) -> String {
            self.address.0.trim_start_matches("0x").to_lowercase()
        }
    }

    /// A `ServerChallenge` bound to `bound`. `server_sign` is a real (if arbitrary) signature
    /// so the struct is well-formed; these tests only exercise the client side of the flow.
    fn server_challenge(bound: &IanColeman, iat: i64) -> ServerChallenge {
        ServerChallenge {
            address: AddressETH(bound.address_norm()),
            iat: Timestamp(iat),
            exp: Timestamp(iat + 300),
            code: Challenge::new("0f1e2d3c4b5a6978".into()),
            server_sign: IanColeman::account0()
                .signer()
                .sign(Challenge::new("server-code".into())),
        }
    }

    fn req(iat: i64, ttl: i64) -> ReqChallenge {
        ReqChallenge::new(
            AddressETH(IanColeman::account0().address_norm()),
            Timestamp(iat),
            Second(ttl),
        )
    }

    #[test]
    fn req_challenge_valid_when_drift_and_ttl_are_in_bounds() {
        let r = req(1_000, 100);
        assert!(r.is_valid(&Timestamp(1_000), &Second(10), &Second(200)));
        assert!(r.is_valid(&Timestamp(1_005), &Second(10), &Second(200)));
    }

    #[test]
    fn req_challenge_rejects_drift_at_or_past_tolerance() {
        let r = req(1_000, 100);
        // `lt` is strict: drift == tolerance is already out
        assert!(!r.is_valid(&Timestamp(1_010), &Second(10), &Second(200)));
        assert!(!r.is_valid(&Timestamp(980), &Second(10), &Second(200)));
    }

    #[test]
    fn req_challenge_rejects_session_ttl_at_or_over_max() {
        assert!(!req(1_000, 200).is_valid(&Timestamp(1_000), &Second(10), &Second(200)));
        assert!(req(1_000, 199).is_valid(&Timestamp(1_000), &Second(10), &Second(200)));
    }

    #[test]
    fn metamask_msg_is_eip191_framed_and_field_sensitive() {
        let acc = IanColeman::account0();
        let sc = server_challenge(&acc, 1_000);

        // pre-image = "\x19Ethereum Signed Message:\n" + body-byte-len + body
        let framed = String::from_utf8(sc.metamask_msg().hex().to_bytes().unwrap()).unwrap();
        let body = sc.to_sign();
        assert_eq!(
            framed,
            format!("\x19Ethereum Signed Message:\n{}{}", body.len(), body)
        );

        // deterministic for the same challenge, different once a field changes
        assert_eq!(sc.metamask_msg().hex(), sc.clone().metamask_msg().hex());
        assert_ne!(
            sc.metamask_msg().hex(),
            server_challenge(&acc, 2_000).metamask_msg().hex()
        );
    }

    #[test]
    fn client_signature_over_metamask_msg_recovers_to_the_bound_address() {
        let acc = IanColeman::account0();
        let sc = server_challenge(&acc, 1_000);

        let answer = acc.signer().sign(sc.metamask_msg());

        let signer = recover(&sc.metamask_msg(), &answer).expect("recovers a key");
        assert_eq!(signer.address().unwrap().0, acc.address_norm());
    }

    #[test]
    fn signature_does_not_recover_to_the_bound_address_for_a_tampered_challenge() {
        let acc = IanColeman::account0();
        let issued = server_challenge(&acc, 1_000);
        let answer = acc.signer().sign(issued.metamask_msg());

        // server verifies against a challenge whose `iat` was altered in transit
        let tampered = server_challenge(&acc, 1_001);
        let mismatched = match recover(&tampered.metamask_msg(), &answer) {
            Err(_) => true,
            Ok(pk) => pk.address().unwrap().0 != acc.address_norm(),
        };
        assert!(mismatched);
    }

    #[test]
    fn signature_from_a_different_wallet_does_not_recover_to_the_bound_address() {
        let bound = IanColeman::account0();
        let sc = server_challenge(&bound, 1_000);

        let answer = IanColeman::account1().signer().sign(sc.metamask_msg());

        let signer = recover(&sc.metamask_msg(), &answer).expect("recovers a key");
        assert_eq!(
            signer.address().unwrap().0,
            IanColeman::account1().address_norm()
        );
        assert_ne!(signer.address().unwrap().0, bound.address_norm());
    }

    #[test]
    fn hexsign_round_trips_through_its_wire_hex() {
        let sig = IanColeman::account0()
            .signer()
            .sign(Challenge::new("x".into()));
        assert_eq!(HexSign::new(sig.hex()).unwrap().hex(), sig.hex());
    }
}
