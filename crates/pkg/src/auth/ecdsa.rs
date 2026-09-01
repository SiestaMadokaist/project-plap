use k256::ecdsa::{
    signature::hazmat::PrehashVerifier, RecoveryId, Signature, SigningKey, VerifyingKey,
};
use serde::{de::Deserializer, Deserialize, Serialize};
use sha3::{Digest, Keccak256};

use crate::{auth::errors::AuthError, types::strings::Hex};

/// strip an optional `0x`/`0X` prefix (`hex::decode` itself rejects one).
fn strip_0x(s: &str) -> &str {
    s.strip_prefix("0x")
        .or_else(|| s.strip_prefix("0X"))
        .unwrap_or(s)
}

fn decode_hex(h: &Hex) -> Result<Vec<u8>, AuthError> {
    hex::decode(strip_0x(&h.0)).map_err(|_| AuthError::Deserialize)
}

/// keccak256 of the challenge's raw UTF-8 bytes - the prehash every `sign`/`verify`/`recover`
/// in this module operates on. Any EIP-191 personal-sign framing is the caller's job (see
/// `dto::resources::login::ServerChallenge::metamask_msg`); this function does not add it.
fn keccak(msg: &Challenge) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(msg.0.as_bytes());
    let mut out = [0u8; 32];
    out.copy_from_slice(&hasher.finalize());
    out
}

#[derive(Serialize, Clone, PartialEq, PartialOrd, Debug)]
pub struct AddressETH(pub String);

impl AddressETH {
    pub fn hex(&self) -> Hex {
        let s = format!("0x{}", self.0);
        Hex(s)
    }
}

/// Normalizes on the way in - `0x`/`0X`-prefixed or not, any case - so an in-memory
/// `AddressETH` is always lowercase with no `0x`, no matter where it came from (a
/// DynamoDB row, wire JSON, a test fixture). [`AddressETH::hex`] re-adds the `0x` only
/// where a display/signing context needs it.
impl<'de> Deserialize<'de> for AddressETH {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Ok(AddressETH(strip_0x(&raw).to_lowercase()))
    }
}

#[derive(Clone)]
pub struct PubKey(VerifyingKey);
#[derive(Clone)]
pub struct PrivKey(SigningKey);

#[derive(Serialize, Deserialize, Clone)]
pub struct Challenge(String);
impl Challenge {
    pub fn new(s: String) -> Self {
        Self(s)
    }

    /// 32 bytes of OS randomness, hex-encoded - a fresh nonce for a server challenge.
    pub fn random() -> Self {
        let mut bytes = [0u8; 32];
        getrandom::fill(&mut bytes).expect("OS RNG must be available");
        Self(hex::encode(bytes))
    }

    /// hex-encoding of the challenge string's bytes - a byte-safe encoding for contexts
    /// that can't tolerate arbitrary characters (e.g. newlines).
    pub fn hex(&self) -> Hex {
        Hex(hex::encode(self.0.as_bytes()))
    }

    /// The raw challenge string, unmodified. Safe to embed directly in a larger message
    /// only where the caller controls its shape - see `ServerChallenge::sign_blob`,
    /// which relies on `Challenge::random`'s output being a plain hex nonce.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl PubKey {
    pub fn wrap(v: VerifyingKey) -> Self {
        Self(v)
    }

    /// SEC1-encoded point - 33-byte compressed (`0x02`/`0x03`) or 65-byte uncompressed
    /// (`0x04`) - as hex, `0x` prefix optional. Not a DER/PKCS8 wrapper.
    pub fn new(hex: Hex) -> Result<Self, AuthError> {
        let bytes = decode_hex(&hex)?;
        VerifyingKey::from_sec1_bytes(&bytes)
            .map(Self)
            .map_err(|_| AuthError::InvalidPubkey)
    }

    /// Check `sign` against this key over `msg`. `Ok(false)` means the signature is
    /// well-formed but doesn't verify; `Err` means it couldn't be parsed at all.
    pub fn verify(&self, msg: Challenge, sign: HexSign) -> Result<bool, AuthError> {
        let sig = sign.get()?;
        Ok(self.0.verify_prehash(&keccak(&msg), &sig).is_ok())
    }

    /// Standard Ethereum address derivation: keccak256 of the uncompressed pubkey's
    /// `X || Y` (i.e. dropping the leading `0x04` tag), last 20 bytes. Lowercase hex,
    /// no `0x`, no EIP-55 checksum - matches how `AddressETH::hex` re-adds the prefix.
    pub fn address(&self) -> Result<AddressETH, AuthError> {
        let point = self.0.to_sec1_point(false);
        let hash = Keccak256::digest(&point.as_bytes()[1..]);
        Ok(AddressETH(hex::encode(&hash[12..])))
    }
}

impl PrivKey {
    pub fn wrap(v: SigningKey) -> Self {
        Self(v)
    }

    /// 32-byte scalar as hex, `0x` prefix optional.
    pub fn new(hex: Hex) -> Result<Self, AuthError> {
        let bytes = decode_hex(&hex)?;
        SigningKey::from_slice(&bytes)
            .map(Self)
            .map_err(|_| AuthError::Deserialize)
    }

    pub fn hex(&self) -> Hex {
        Hex(format!("0x{}", hex::encode(self.0.to_bytes())))
    }

    /// The matching public key.
    pub fn pubkey(&self) -> PubKey {
        PubKey(*self.0.verifying_key())
    }

    /// Sign the keccak256 prehash of `msg`, yielding a 65-byte `r || s || v` signature
    /// with `v` in {27, 28} - the same layout a wallet's `personal_sign` returns, so the
    /// server's own challenge signatures are recoverable the same way client ones are.
    pub fn sign(&self, msg: Challenge) -> HexSign {
        let (sig, rec) = self.0.sign_prehash_recoverable(&keccak(&msg));
        HexSign::wrap(sig, rec)
    }
}

#[derive(Clone, Serialize, Debug, Deserialize)]
pub struct HexSign(Hex);

impl HexSign {
    /// Pack `r || s` (64 bytes) plus the recovery id, biased to Ethereum's `v` of 27/28.
    pub fn wrap(sig: Signature, rec: RecoveryId) -> Self {
        let mut bytes = sig.to_bytes().to_vec();
        bytes.push(rec.to_byte() + 27);
        Self(Hex(format!("0x{}", hex::encode(bytes))))
    }

    /// Parse a 65-byte `r || s || v` signature hex, `0x` prefix optional.
    pub fn new(hex: Hex) -> Result<Self, AuthError> {
        let bytes = decode_hex(&hex)?;
        if bytes.len() != 65 {
            return Err(AuthError::InvalidSignature);
        }
        Signature::from_slice(&bytes[..64]).map_err(|_| AuthError::InvalidSignature)?;
        Self::recovery_byte(bytes[64])?;
        Ok(Self(hex))
    }

    pub fn hex(&self) -> Hex {
        self.0.clone()
    }

    /// The `r || s` component as a verifiable signature (the trailing `v` is dropped -
    /// it's only needed to *recover* a key, see [`recover`]).
    pub fn get(&self) -> Result<Signature, AuthError> {
        let bytes = decode_hex(&self.0)?;
        let rs = bytes.get(..64).ok_or(AuthError::InvalidSignature)?;
        Signature::from_slice(rs).map_err(|_| AuthError::InvalidSignature)
    }

    /// `(r||s signature, recovery id)` - everything [`recover`] needs.
    fn parts(&self) -> Result<(Signature, RecoveryId), AuthError> {
        let bytes = decode_hex(&self.0)?;
        if bytes.len() != 65 {
            return Err(AuthError::InvalidSignature);
        }
        let sig = Signature::from_slice(&bytes[..64]).map_err(|_| AuthError::InvalidSignature)?;
        Ok((sig, Self::recovery_byte(bytes[64])?))
    }

    /// Accept both the Ethereum `v` (27/28) and a bare 0/1 recovery id.
    fn recovery_byte(v: u8) -> Result<RecoveryId, AuthError> {
        let raw = if v >= 27 { v - 27 } else { v };
        RecoveryId::from_byte(raw).ok_or(AuthError::InvalidSignature)
    }
}

/// Recover the signer's public key from a 65-byte signature over `msg`. This is how the
/// login flow gets a key to check `client_sign` against when it only knows the claimed
/// address: recover, then compare [`PubKey::address`] to the expected one.
pub fn recover(msg: &Challenge, sign: &HexSign) -> Result<PubKey, AuthError> {
    let (sig, rec) = sign.parts()?;
    VerifyingKey::recover_from_prehash(&keccak(msg), &sig, rec)
        .map(PubKey)
        .map_err(|_| AuthError::VerificationFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hardhat/Ganache well-known dev account 0 ("test test ... junk", m/44'/60'/0'/0/0) -
    /// public, deterministic, never holds funds. Address verified against hardhat.org.
    const ACCOUNT_0_PRIVKEY: &str =
        "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    const ACCOUNT_0_ADDRESS: &str = "f39fd6e51aad88f6f4ce6ab8827279cfffb92266";

    fn privkey() -> PrivKey {
        PrivKey::new(Hex(ACCOUNT_0_PRIVKEY.into())).expect("valid scalar")
    }

    #[test]
    fn address_derivation_matches_known_account() {
        let signer = SigningKey::from_slice(&hex::decode(ACCOUNT_0_PRIVKEY).unwrap()).unwrap();
        let pubkey = PubKey::wrap(*signer.verifying_key());
        assert_eq!(pubkey.address().unwrap().0, ACCOUNT_0_ADDRESS);
    }

    #[test]
    fn sign_then_recover_roundtrips_to_the_signer() {
        let msg = Challenge::new("\x19Ethereum Signed Message:\n5hello".into());
        let sig = privkey().sign(msg.clone());

        let recovered = recover(&msg, &sig).expect("recovers");
        assert_eq!(recovered.address().unwrap().0, ACCOUNT_0_ADDRESS);
    }

    #[test]
    fn verify_accepts_own_signature_and_rejects_a_different_message() {
        let signer = SigningKey::from_slice(&hex::decode(ACCOUNT_0_PRIVKEY).unwrap()).unwrap();
        let pubkey = PubKey::wrap(*signer.verifying_key());

        let signed = Challenge::new("code-123".into());
        let sig = privkey().sign(signed.clone());

        assert!(pubkey.verify(signed, sig.clone()).unwrap());
        assert!(!pubkey
            .verify(Challenge::new("code-124".into()), sig)
            .unwrap());
    }

    #[test]
    fn hexsign_survives_a_hex_round_trip() {
        let sig = privkey().sign(Challenge::new("x".into()));
        let reparsed = HexSign::new(sig.hex()).expect("well-formed");
        assert_eq!(reparsed.hex(), sig.hex());
    }

    #[test]
    fn new_rejects_wrong_length_signature() {
        let short = Hex(format!("0x{}", "aa".repeat(10)));
        assert!(matches!(
            HexSign::new(short),
            Err(AuthError::InvalidSignature)
        ));
    }

    #[test]
    fn privkey_hex_round_trips() {
        let hex = privkey().hex();
        let again = PrivKey::new(hex.clone()).unwrap();
        assert_eq!(again.hex(), hex);
    }
}
