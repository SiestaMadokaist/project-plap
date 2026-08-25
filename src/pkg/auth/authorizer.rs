use k256::ecdsa::{signature::hazmat::PrehashVerifier, Signature, VerifyingKey};
use sha3::{Digest, Keccak256};

use crate::pkg::auth::{
    authreq::{AuthReq, AuthSecret},
    claims::Claims,
    errors::AuthError,
};

/// how long an issued session token stays valid for.
const SESSION_TTL_SECS: i64 = 60 * 60 * 24;

pub struct Authorizer<D> {
    secret: AuthSecret,
    _p: std::marker::PhantomData<D>,
}

impl<D> Authorizer<D> {
    pub fn authenticate(&self, req: AuthReq) -> Result<String, AuthError> {
        let pubkey_bytes = req.pubkey.to_bytes().map_err(|_| AuthError::Deserialize)?;
        // raw SEC1-encoded point (0x02/0x03 compressed or 0x04 uncompressed), not a DER/PKCS8
        // wrapper - that's the format a wallet's pubkey comes in, not an X.509 key.
        let verifying_key =
            VerifyingKey::from_sec1_bytes(&pubkey_bytes).map_err(|_| AuthError::InvalidPubkey)?;

        let sig_bytes = req
            .signature
            .to_bytes()
            .map_err(|_| AuthError::Deserialize)?;
        // wallets (personal_sign/eth_sign) return a 65-byte r||s||v signature; the trailing
        // 1-byte recovery id (v) is only needed to *recover* a pubkey from the signature. We
        // already have the pubkey from the request, so we only verify against r||s.
        let sig_bytes = match sig_bytes.len() {
            65 => &sig_bytes[..64],
            64 => &sig_bytes[..],
            _ => return Err(AuthError::InvalidSignature),
        };
        let signature =
            Signature::from_slice(sig_bytes).map_err(|_| AuthError::InvalidSignature)?;

        // Wallets don't sign the raw message - personal_sign/eth_sign hash
        // keccak256("\x19Ethereum Signed Message:\n{len}{msg}") first (EIP-191). Skipping this
        // prefix is the single most common reason "correct" signatures fail to verify.
        let msg = req.msg();
        let prefixed = format!("\x19Ethereum Signed Message:\n{}{}", msg.len(), msg);
        let digest = Keccak256::digest(prefixed.as_bytes());

        // "hazmat" because verify_prehash trusts the caller to have hashed the message
        // correctly - unlike `Verifier::verify`, it does no hashing of its own, so getting the
        // digest above wrong fails silently (wrong signature) rather than as a compile error.
        verifying_key
            .verify_prehash(&digest, &signature)
            .map_err(|_| AuthError::VerificationFailed)?;

        let address = eth_address(&verifying_key);
        let now = chrono::Utc::now().timestamp();
        let claims = Claims {
            sub: address,
            iat: now,
            exp: now + SESSION_TTL_SECS,
        };

        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AuthError::TokenIssue(e.to_string()))
    }

    pub fn authorize(&self, _jwt: &str) -> Result<D, AuthError> {
        Err(AuthError::TODO)
    }
}

/// standard Ethereum address derivation: keccak256 of the uncompressed pubkey's X||Y
/// coordinates (i.e. without the leading 0x04 tag byte), last 20 bytes, 0x-prefixed.
fn eth_address(key: &VerifyingKey) -> String {
    let uncompressed = key.to_sec1_point(false);
    let hash = Keccak256::digest(&uncompressed.as_bytes()[1..]);
    format!("0x{}", hex::encode(&hash[12..]))
}
