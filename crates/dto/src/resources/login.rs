use std::ops::{Add, Sub};

use chrono::Utc;
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
        match iat.utc() {
            // if iat is invalid, i guess we just return invalid challenge
            None => Self {
                address,
                iat: Timestamp(1),
                exp: Timestamp(0),
            },
            Some(x) => {
                let exp = x.add(ttl.to_delta());
                Self {
                    address,
                    iat,
                    exp: exp.into(),
                }
            }
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
        server_time: &chrono::DateTime<Utc>,
        tolerance: &Second,
        max_session_ttl: &Second,
    ) -> bool {
        let iat = self.iat.utc();
        // client time's drift must be within tolerance from server time
        let validation1 = match &iat {
            None => false,
            Some(x) => x.sub(server_time).abs().lt(&tolerance.to_delta()),
        };
        let exp = self.exp.utc();
        let validation2 = match &exp {
            None => false,
            Some(x) => x.gt(server_time),
        };
        let validation3 = match (&exp, &iat) {
            (None, _) => false,
            (_, None) => false,
            (Some(e), Some(i)) => e.sub(i).lt(&max_session_ttl.to_delta()),
        };
        validation1 && validation2 && validation3
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

    /// What the wallet is actually asked to sign. Deliberately readable (unlike a
    /// `-`-joined blob of raw fields) so a user has something to sanity-check before
    /// approving; still fully binding on the exact `iat`/`exp` seconds (via lossless
    /// second-precision ISO-8601, not just a date) so tampering after the server's
    /// `server_sign` still fails `verify_issued_by`.
    fn sign_blob(address: &AddressETH, iat: Timestamp, exp: Timestamp, code: &Challenge) -> String {
        format!(
            "Sign in to Project-Plap\n\nAddress: {}\nNonce: {}\nIssued At: {}\nExpires At: {}",
            address.hex(),
            code.as_str(),
            Self::fmt_time(iat),
            Self::fmt_time(exp),
        )
    }

    /// Second-precision ISO-8601 rendering of a `Timestamp`, chosen because it's
    /// lossless against `Timestamp`'s own precision (no two distinct seconds render the
    /// same). Falls back to the raw integer on the unreachable out-of-range case rather
    /// than risk collapsing two different instants to one string.
    fn fmt_time(t: Timestamp) -> String {
        match t.utc() {
            Some(dt) => dt.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            None => t.0.to_string(),
        }
    }

    fn to_sign(&self) -> String {
        Self::sign_blob(&self.address, self.iat, self.exp, &self.code)
    }

    /// The raw, *unframed* message the client's wallet signs (see [`Self::sign_blob`]
    /// for its shape). A wallet's `personal_sign` applies the EIP-191 frame itself, so
    /// the frontend hands this string straight to the wallet - it must not pre-frame
    /// it. Server-side, [`Self::metamask_msg`] is exactly the EIP-191 framing of this
    /// same string (see the `metamask_msg_is_eip191_of_sign_message` test), which is
    /// what `recover`/`verify` operate on.
    pub fn sign_message(&self) -> String {
        self.to_sign()
    }

    /// EIP-191 personal-sign framing of [`Self::sign_message`]: what a wallet's
    /// personal_sign/eth_sign actually hashes and signs. Used server-side for recovery
    /// and verification; the client never builds this itself (its wallet does).
    pub fn metamask_msg(&self) -> Challenge {
        let msg = self.sign_message();
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

    /// `is_valid` takes wall-clock server time, not a `Timestamp`.
    fn server_time(secs: i64) -> chrono::DateTime<chrono::Utc> {
        Timestamp(secs).utc().expect("in-range unix seconds")
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
        assert!(r.is_valid(&server_time(1_000), &Second(10), &Second(200)));
        assert!(r.is_valid(&server_time(1_005), &Second(10), &Second(200)));
    }

    #[test]
    fn req_challenge_rejects_drift_at_or_past_tolerance() {
        let r = req(1_000, 100);
        // `lt` is strict: drift == tolerance is already out
        assert!(!r.is_valid(&server_time(1_010), &Second(10), &Second(200)));
        assert!(!r.is_valid(&server_time(980), &Second(10), &Second(200)));
    }

    #[test]
    fn req_challenge_rejects_session_ttl_at_or_over_max() {
        assert!(!req(1_000, 200).is_valid(&server_time(1_000), &Second(10), &Second(200)));
        assert!(req(1_000, 199).is_valid(&server_time(1_000), &Second(10), &Second(200)));
    }

    #[test]
    fn metamask_msg_is_eip191_of_sign_message() {
        let sc = server_challenge(&IanColeman::account0(), 1_000);

        // the invariant the frontend relies on: `metamask_msg` is precisely the EIP-191
        // frame around the public `sign_message`, so a wallet signing `sign_message`
        // (which frames it once) matches what the server recovers from `metamask_msg`.
        let framed = String::from_utf8(sc.metamask_msg().hex().to_bytes().unwrap()).unwrap();
        let body = sc.sign_message();
        assert_eq!(
            framed,
            format!("\x19Ethereum Signed Message:\n{}{}", body.len(), body)
        );
    }

    #[test]
    fn metamask_msg_is_eip191_framed_and_field_sensitive() {
        let acc = IanColeman::account0();
        let sc = server_challenge(&acc, 1_000);

        // pre-image = "\x19Ethereum Signed Message:\n" + body-byte-len + body
        let framed = String::from_utf8(sc.metamask_msg().hex().to_bytes().unwrap()).unwrap();
        let body = sc.sign_message();
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
