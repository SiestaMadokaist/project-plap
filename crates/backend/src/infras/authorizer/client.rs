use std::ops::Add;

use async_trait::async_trait;
use domain::errors::DomainError;
use dto::resources::login::{ClientAnswer, ReqChallenge, ServerChallenge};
use pkg::{
    auth::{
        claims::{AuthClaims, Username, JWT},
        errors::AuthError,
    },
    types::{
        crypto::EthPriv,
        time::{Second, Timestamp},
    },
};

use crate::application::ports::clients::authorizer::Authorizer;
use crate::infras::authorizer::ecdsa::{eip191, random_code, recover, PrivKey, PubKey};

/// Stateless Ethereum-wallet authenticator.
///
/// - `challenge` mints a short-lived `ServerChallenge`, signed with `privkey` so no
///   server-side session store is needed.
/// - `answer` verifies that signature, recovers the wallet from the client's answer, and
///   issues an HS256 JWT keyed by `secret`.
/// - `validate` checks that JWT.
#[derive(Debug)]
pub struct EthAuth {
    /// HS256 key for session tokens.
    secret: String,
    /// server key that signs challenges.
    privkey: PrivKey,
    /// cached `privkey.pubkey()`, used to verify a returned challenge.
    server_pub: PubKey,
    /// how long a minted challenge stays answerable.
    challenge_ttl: Second,
    /// lifetime of an issued session token.
    session_ttl: Second,
    /// allowed drift between the client's clock and ours when validating a request.
    clock_skew: Second,
    /// tokens issued before this `iat` are rejected even if unexpired (forced re-login).
    min_iat: Timestamp,
}

impl EthAuth {
    pub fn new(
        secret: String,
        privkey: EthPriv,
        challenge_ttl: Second,
        session_ttl: Second,
        clock_skew: Second,
        min_iat: Timestamp,
    ) -> Result<Self, AuthError> {
        let privkey = PrivKey::new(&privkey)?;
        let server_pub = privkey.pubkey();
        Ok(Self {
            secret,
            privkey,
            server_pub,
            challenge_ttl,
            session_ttl,
            clock_skew,
            min_iat,
        })
    }
}

/// every auth failure surfaces as a 403 (`NotAllowed`) - the caller shouldn't learn
/// which specific check tripped.
fn deny(e: impl ToString) -> DomainError {
    DomainError::NotAllowed(e.to_string())
}

#[async_trait]
impl Authorizer for EthAuth {
    async fn challenge(&self, req: ReqChallenge) -> Result<ServerChallenge, DomainError> {
        let now = Timestamp::now().utc().expect("should be a valid utc");
        if !req.is_valid(&now, &self.clock_skew, &self.session_ttl) {
            return Err(deny("challenge request failed validation"));
        }
        let address = req.address().clone();
        let (iat, exp): (Timestamp, Timestamp) =
            (req.iat(), now.add(self.challenge_ttl.to_delta()).into());
        let code = random_code();
        // signed raw (not EIP-191 framed): only we ever verify this one, no wallet involved.
        let blob = ServerChallenge::sign_blob(&address, iat, exp, &code);
        let server_sign = self.privkey.sign(blob.as_bytes());
        Ok(ServerChallenge::new(address, iat, exp, code, server_sign))
    }

    async fn answer(&self, username: Username, ans: ClientAnswer) -> Result<JWT, DomainError> {
        let sc = &ans.challenge;

        // 1. this is a challenge we minted, with its fields intact
        let issued_by_us = self
            .server_pub
            .verify(sc.sign_message().as_bytes(), sc.server_sign())
            .unwrap_or(false);
        if !issued_by_us {
            return Err(deny("challenge signature is not ours"));
        }
        // 2. it hasn't gone stale
        let now = chrono::Utc::now();
        if sc.exp().0 <= now.timestamp() {
            return Err(deny("challenge has expired"));
        }
        // 3. the answer was signed by the wallet the challenge is bound to
        // the wallet signed the EIP-191 framing of the plain text; recover against the same.
        let framed = eip191(&sc.sign_message());
        let signer = recover(&framed, &ans.client_sign).map_err(deny)?;
        if signer.address().map_err(deny)?.get().s != sc.address().get().s {
            return Err(deny("answer was signed by a different address"));
        }

        // 4. issue the session token
        let claims = AuthClaims {
            username,
            sub: sc.address().get().s.clone(),
            iat: now.into(),
            exp: now.add(self.session_ttl.to_delta()).into(),
        };
        JWT::sign(&claims, self.secret.as_bytes()).map_err(deny)
    }

    async fn validate(&self, jwt: JWT) -> Result<AuthClaims, DomainError> {
        let claims = jwt.decode(self.secret.as_bytes()).map_err(deny)?;
        if claims.iat.0 < self.min_iat.0 {
            return Err(deny("token predates the revocation cutoff"));
        }
        Ok(claims)
    }
}

#[cfg(test)]
mod tests {
    use pkg::types::crypto::{Eth0x, EthSign};

    use super::*;

    // iancoleman "test test ... junk" m/44'/60'/0'/0/{0,1}
    const CLIENT_PRIVKEY: &str = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    const CLIENT_ADDRESS: &str = "f39fd6e51aad88f6f4ce6ab8827279cfffb92266";
    const SERVER_PRIVKEY: &str = "59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

    fn username() -> Username {
        Username("username".into())
    }

    fn priv_of(hex: &str) -> EthPriv {
        EthPriv::try_from(hex).unwrap()
    }

    fn client_address() -> Eth0x {
        Eth0x::eth(&format!("0x{CLIENT_ADDRESS}")).unwrap()
    }

    fn eth(challenge_ttl: i64, min_iat: i64) -> EthAuth {
        EthAuth::new(
            "hs256-test-secret".into(),
            priv_of(SERVER_PRIVKEY),
            Second(challenge_ttl),
            Second(86_400),
            Second(60),
            Timestamp(min_iat),
        )
        .expect("valid server key")
    }

    fn client() -> PrivKey {
        PrivKey::new(&priv_of(CLIENT_PRIVKEY)).unwrap()
    }

    fn req() -> ReqChallenge {
        ReqChallenge::new(
            client_address(),
            Timestamp::now(),
            Second(3_600),
        )
    }

    fn answer_with(signer: &PrivKey, challenge: ServerChallenge) -> ClientAnswer {
        // what a wallet's personal_sign does: frame, hash, sign.
        let client_sign: EthSign = signer.sign(&eip191(&challenge.sign_message()));
        ClientAnswer {
            challenge,
            client_sign,
        }
    }

    #[tokio::test]
    async fn full_handshake_issues_a_token_for_the_signing_wallet() {
        let auth = eth(300, 0);
        let sc = auth.challenge(req()).await.expect("challenge minted");
        let jwt = auth
            .answer(username(), answer_with(&client(), sc))
            .await
            .expect("answer accepted");
        let claims = auth.validate(jwt).await.expect("token validates");

        assert_eq!(claims.sub, format!("0x{}", CLIENT_ADDRESS));
        assert!(claims.exp.0 > claims.iat.0);
    }

    #[tokio::test]
    async fn answer_signed_by_a_different_wallet_is_rejected() {
        let auth = eth(300, 0);
        let sc = auth.challenge(req()).await.unwrap();

        let intruder = PrivKey::new(&priv_of(SERVER_PRIVKEY)).unwrap();
        let out = auth.answer(username(), answer_with(&intruder, sc)).await;

        assert!(matches!(out, Err(DomainError::NotAllowed(_))));
    }

    #[tokio::test]
    async fn a_challenge_from_another_server_is_rejected() {
        let ours = eth(300, 0);
        let theirs = EthAuth::new(
            "other".into(),
            priv_of(CLIENT_PRIVKEY),
            Second(300),
            Second(86_400),
            Second(60),
            Timestamp(0),
        )
        .unwrap();

        let foreign = theirs.challenge(req()).await.unwrap();
        let out = ours
            .answer(username(), answer_with(&client(), foreign))
            .await;

        assert!(matches!(out, Err(DomainError::NotAllowed(_))));
    }

    #[tokio::test]
    async fn an_expired_challenge_is_rejected() {
        let auth = eth(0, 0); // challenge is stale the instant it's minted
        let sc = auth.challenge(req()).await.unwrap();

        let out = auth.answer(username(), answer_with(&client(), sc)).await;
        assert!(matches!(out, Err(DomainError::NotAllowed(_))));
    }

    #[tokio::test]
    async fn validate_rejects_a_token_issued_before_min_iat() {
        let issuer = eth(300, 0);
        let sc = issuer.challenge(req()).await.unwrap();
        let jwt = issuer
            .answer(username(), answer_with(&client(), sc))
            .await
            .unwrap();

        // same secret, but a cutoff in the future
        let strict = eth(300, Timestamp::now().0 + 3_600);
        assert!(matches!(
            strict.validate(jwt).await,
            Err(DomainError::NotAllowed(_))
        ));
    }

    #[tokio::test]
    async fn validate_rejects_a_tampered_token() {
        let auth = eth(300, 0);
        let sc = auth.challenge(req()).await.unwrap();
        let JWT(mut raw) = auth
            .answer(username(), answer_with(&client(), sc))
            .await
            .unwrap();
        raw.pop();
        raw.push(if raw.ends_with('a') { 'b' } else { 'a' });

        assert!(auth.validate(JWT(raw)).await.is_err());
    }

    #[tokio::test]
    async fn challenge_rejects_a_request_with_too_much_clock_skew() {
        let auth = eth(300, 0);
        let skewed = ReqChallenge::new(
            client_address(),
            Timestamp(Timestamp::now().0 + 3_600),
            Second(3_600),
        );
        assert!(auth.challenge(skewed).await.is_err());
    }
}
