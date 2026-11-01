use k256::ecdsa::{
    signature::hazmat::PrehashVerifier, RecoveryId, Signature, SigningKey, VerifyingKey,
};
use pkg::{
    auth::errors::AuthError,
    types::{
        crypto::{Eth0x, EthPriv, EthPub, EthSign},
        hex::{zx, Hex, Zx},
    },
};
use sha3::{Digest, Keccak256};

/// Ethereum encodes the recovery id as `v = 27 + recid`.
const V_OFFSET: u8 = 27;

/// `0x`-prefixed hex string -> bytes (the `0x` is optional, `hex::decode` itself rejects one).
fn decode_hex(h: &Hex<Zx>) -> Result<Vec<u8>, AuthError> {
    let s = h.s.strip_prefix("0x").unwrap_or(&h.s);
    hex::decode(s).map_err(|_| AuthError::Deserialize)
}

/// keccak256 of the raw message bytes - the prehash every `sign`/`verify`/`recover` in this
/// module operates on. Any EIP-191 personal-sign framing is the caller's job (see [`eip191`]);
/// the signing functions here do not add it.
fn keccak(msg: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    out.copy_from_slice(&Keccak256::digest(msg));
    out
}

/// EIP-191 personal-sign framing: what a wallet's `personal_sign` actually hashes and signs.
/// A wallet applies this itself, so it is only needed server-side, to recover/verify a wallet's
/// signature over the plain challenge text.
pub fn eip191(msg: &str) -> Vec<u8> {
    format!("\x19Ethereum Signed Message:\n{}{}", msg.len(), msg).into_bytes()
}

/// 32 bytes of OS randomness, `0x`-hex-encoded - a fresh nonce for a server challenge.
pub fn random_code() -> Hex<Zx> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("OS RNG must be available");
    zx(&hex::encode(bytes)).expect("hex::encode output is valid hex")
}

#[derive(Clone, Debug)]
pub struct PubKey(VerifyingKey);
#[derive(Clone, Debug)]
pub struct PrivKey(SigningKey);

impl PubKey {
    /// SEC1-encoded point - 33-byte compressed (`0x02`/`0x03`) or 65-byte uncompressed
    /// (`0x04`). Not a DER/PKCS8 wrapper.
    pub fn new(key: &EthPub) -> Result<Self, AuthError> {
        let bytes = decode_hex(key.as_hex())?;
        VerifyingKey::from_sec1_bytes(&bytes)
            .map(Self)
            .map_err(|_| AuthError::InvalidPubkey)
    }

    /// Check `sign` against this key over `msg`. `Ok(false)` means the signature is
    /// well-formed but doesn't verify; `Err` means it couldn't be parsed at all.
    pub fn verify(&self, msg: &[u8], sign: &EthSign) -> Result<bool, AuthError> {
        let (sig, _) = split(sign)?;
        Ok(self.0.verify_prehash(&keccak(msg), &sig).is_ok())
    }

    /// Standard Ethereum address derivation: keccak256 of the uncompressed pubkey's
    /// `X || Y` (i.e. dropping the leading `0x04` tag), last 20 bytes. `0x` + lowercase hex,
    /// no EIP-55 checksum.
    pub fn address(&self) -> Result<Eth0x, AuthError> {
        let point = self.0.to_sec1_point(false);
        let hash = Keccak256::digest(&point.as_bytes()[1..]);
        Eth0x::eth(&format!("0x{}", hex::encode(&hash[12..]))).map_err(|_| AuthError::Deserialize)
    }
}

impl PrivKey {
    /// 32-byte scalar.
    pub fn new(key: &EthPriv) -> Result<Self, AuthError> {
        let bytes = decode_hex(key.as_hex())?;
        SigningKey::from_slice(&bytes)
            .map(Self)
            .map_err(|_| AuthError::Deserialize)
    }

    /// The matching public key.
    pub fn pubkey(&self) -> PubKey {
        PubKey(*self.0.verifying_key())
    }

    /// Sign the keccak256 prehash of `msg`, yielding a 65-byte `r || s || v` signature
    /// with `v` in {27, 28} - the same layout a wallet's `personal_sign` returns, so the
    /// server's own challenge signatures are recoverable the same way client ones are.
    pub fn sign(&self, msg: &[u8]) -> EthSign {
        let (sig, rec) = self.0.sign_prehash_recoverable(&keccak(msg));
        let mut bytes = sig.to_bytes().to_vec();
        bytes.push(rec.to_byte() + V_OFFSET);
        EthSign::try_from(format!("0x{}", hex::encode(bytes)).as_str())
            .expect("hex::encode output is valid hex")
    }
}

/// `(r||s signature, recovery id)` out of a 65-byte `r || s || v` signature. Accepts both
/// the Ethereum `v` (27/28) and a bare 0/1 recovery id.
fn split(sign: &EthSign) -> Result<(Signature, RecoveryId), AuthError> {
    let bytes = decode_hex(sign.as_hex())?;
    if bytes.len() != 65 {
        return Err(AuthError::InvalidSignature);
    }
    let sig = Signature::from_slice(&bytes[..64]).map_err(|_| AuthError::InvalidSignature)?;
    let v = bytes[64];
    let raw = if v >= V_OFFSET { v - V_OFFSET } else { v };
    let rec = RecoveryId::from_byte(raw).ok_or(AuthError::InvalidSignature)?;
    Ok((sig, rec))
}

/// Recover the signer's public key from a 65-byte signature over `msg`. This is how the
/// login flow gets a key to check `client_sign` against when it only knows the claimed
/// address: recover, then compare [`PubKey::address`] to the expected one.
pub fn recover(msg: &[u8], sign: &EthSign) -> Result<PubKey, AuthError> {
    let (sig, rec) = split(sign)?;
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
    const ACCOUNT_0_ADDRESS: &str = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266";

    fn privkey() -> PrivKey {
        let key = EthPriv::try_from(ACCOUNT_0_PRIVKEY).unwrap();
        PrivKey::new(&key).expect("valid scalar")
    }

    #[test]
    fn address_derivation_matches_known_account() {
        let address = privkey().pubkey().address().unwrap();
        assert_eq!(address.get().s, ACCOUNT_0_ADDRESS);
    }

    #[test]
    fn sign_then_recover_roundtrips_to_the_signer() {
        let msg = eip191("hello");
        let sig = privkey().sign(&msg);

        let recovered = recover(&msg, &sig).expect("recovers");
        assert_eq!(recovered.address().unwrap().get().s, ACCOUNT_0_ADDRESS);
    }

    #[test]
    fn verify_accepts_own_signature_and_rejects_a_different_message() {
        let pubkey = privkey().pubkey();
        let sig = privkey().sign(b"code-123");

        assert!(pubkey.verify(b"code-123", &sig).unwrap());
        assert!(!pubkey.verify(b"code-124", &sig).unwrap());
    }

    #[test]
    fn recover_rejects_wrong_length_signature() {
        let short = EthSign::try_from(format!("0x{}", "aa".repeat(10)).as_str()).unwrap();
        assert!(matches!(
            recover(b"x", &short),
            Err(AuthError::InvalidSignature)
        ));
    }

    #[test]
    fn random_code_is_32_bytes_of_zx_hex_and_not_repeated() {
        let (a, b) = (random_code(), random_code());
        assert_eq!(a.s.len(), 2 + 64);
        assert_ne!(a.s, b.s);
    }
}
