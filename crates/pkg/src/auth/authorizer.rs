use k256::ecdsa::{signature::hazmat::PrehashVerifier, Signature, VerifyingKey};
use sha3::{Digest, Keccak256};

use crate::{
    auth::{
        authreq::{AuthReq, AuthSecret},
        claims::Claims,
        errors::AuthError,
    },
    types::strings::JWT,
};

/// how long an issued session token stays valid for.
const SESSION_TTL_SECS: i64 = 60 * 60 * 24;

pub struct Authorizer<D> {
    secret: AuthSecret,
    _p: std::marker::PhantomData<D>,
}

impl<D> Authorizer<D> {
    pub fn authenticate(&self, req: AuthReq) -> Result<JWT, AuthError> {
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

        // Wallets don't sign the raw message - personal_sign/eth_sign hash the EIP-191-framed
        // form first (see AuthReq::prefixed). Skipping this prefix is the single most common
        // reason "correct" signatures fail to verify.
        let digest = Keccak256::digest(req.prefixed().as_bytes());

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

        let encoded = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AuthError::TokenIssue(e.to_string()))?;
        Ok(JWT(encoded))
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

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use k256::ecdsa::{signature::hazmat::PrehashSigner, SigningKey};

    use super::*;
    use crate::types::{strings::Hex, time::Timestamp};

    fn authorizer() -> Authorizer<()> {
        Authorizer {
            secret: AuthSecret::new("test-secret"),
            _p: PhantomData,
        }
    }

    /// signs an already-EIP-191-prefixed string, e.g. `AuthReq::prefixed()`'s output - so tests
    /// go through the exact same framing `authenticate` verifies against, not a copy of it.
    fn sign(signing_key: &SigningKey, prefixed: &str) -> String {
        let digest = Keccak256::digest(prefixed.as_bytes());
        let signature: Signature = signing_key
            .sign_prehash(&digest)
            .expect("signing a well-formed digest cannot fail");
        hex::encode(signature.to_bytes())
    }

    fn req(signing_key: &SigningKey, pubkey: &SigningKey, nonce: u32, client_time: i64) -> AuthReq {
        // signature left blank while we compute `prefixed()`, which doesn't depend on it -
        // filled in below once we can actually sign something.
        let mut req = AuthReq {
            pubkey: Hex(hex::encode(
                pubkey.verifying_key().to_sec1_point(false).as_bytes(),
            )),
            signature: Hex(String::new()),
            nonce,
            client_time: Timestamp(client_time),
        };
        req.signature = Hex(sign(signing_key, &req.prefixed()));
        req
    }

    #[test]
    fn authenticate_valid_signature_issues_token() {
        let key = SigningKey::from_slice(&[0x11u8; 32]).expect("valid scalar");
        let req = req(&key, &key, 42, 1_700_000_000);

        let result = authorizer().authenticate(req);

        assert!(result.is_ok(), "expected Ok, got {:?}", result.err());
    }

    #[test]
    fn authenticate_rejects_signature_from_a_different_key_than_the_claimed_pubkey() {
        let signer = SigningKey::from_slice(&[0x11u8; 32]).expect("valid scalar");
        let claimed = SigningKey::from_slice(&[0x22u8; 32]).expect("valid scalar");
        // signed by `signer`, but the request claims `claimed`'s pubkey
        let req = req(&signer, &claimed, 42, 1_700_000_000);

        let result = authorizer().authenticate(req);

        assert!(matches!(result, Err(AuthError::VerificationFailed)));
    }

    #[test]
    fn authenticate_rejects_non_hex_pubkey() {
        let req = AuthReq {
            pubkey: Hex("not-hex".into()),
            signature: Hex("00".repeat(65)),
            nonce: 1,
            client_time: Timestamp(1_700_000_000),
        };

        let result = authorizer().authenticate(req);

        assert!(matches!(result, Err(AuthError::Deserialize)));
    }

    #[test]
    fn authenticate_rejects_wrong_length_signature() {
        let key = SigningKey::from_slice(&[0x11u8; 32]).expect("valid scalar");
        let mut req = req(&key, &key, 42, 1_700_000_000);
        req.signature = Hex("aa".repeat(10)); // 10 bytes, neither 64 nor 65

        let result = authorizer().authenticate(req);

        assert!(matches!(result, Err(AuthError::InvalidSignature)));
    }

    /// Hardhat/Ganache's well-known default dev mnemonic, account 0
    /// ("test test test test test test test test test test test junk", m/44'/60'/0'/0/0) -
    /// public, deterministic, never holds real funds. Verified independently against
    /// https://hardhat.org rather than derived here, so this also checks `eth_address`'s
    /// derivation against a value this code didn't produce itself.
    const HARDHAT_ACCOUNT_0_PRIVATE_KEY: &str =
        "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    const HARDHAT_ACCOUNT_0_ADDRESS: &str = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266";

    #[test]
    fn authenticate_with_hardhat_test_mnemonic_account_0() {
        let key_bytes = hex::decode(HARDHAT_ACCOUNT_0_PRIVATE_KEY).expect("valid hex");
        let key = SigningKey::from_slice(&key_bytes).expect("valid scalar");

        // sanity check the fixture itself before trusting the assertion below - if this
        // fails, the private key and address above don't actually match each other.
        assert_eq!(eth_address(key.verifying_key()), HARDHAT_ACCOUNT_0_ADDRESS);

        let req = req(&key, &key, 7, 1_700_000_000);
        let jwt = authorizer()
            .authenticate(req)
            .expect("valid signature should authenticate");

        let claims = jsonwebtoken::decode::<Claims>(
            &jwt.0,
            &jsonwebtoken::DecodingKey::from_secret(AuthSecret::new("test-secret").as_bytes()),
            &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256),
        )
        .expect("token issued by authenticate must decode with the same secret")
        .claims;

        assert_eq!(claims.sub, HARDHAT_ACCOUNT_0_ADDRESS);
    }
}
