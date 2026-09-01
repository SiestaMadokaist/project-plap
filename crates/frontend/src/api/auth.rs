use domain::errors::DomainError;
use dto::{
    resources::login::{ClientAnswer, LoginResponse, ReqChallenge, ServerChallenge},
    response::{Response, DTO},
};
use gloo_net::http::Request;
use pkg::types::strings::URL;
use serde::Serialize;

/// The two public endpoints of the wallet-login handshake.
pub struct AuthApi {
    host: URL,
}

impl AuthApi {
    pub fn new(host: URL) -> Self {
        Self { host }
    }

    /// Step 1 - ask the server to mint a signed challenge for `req.address`.
    pub async fn request_challenge(
        &self,
        req: &ReqChallenge,
    ) -> Result<ServerChallenge, DomainError> {
        self.post("/users/challenge", req).await
    }

    /// Step 2 - return the wallet-signed challenge and receive a session token.
    pub async fn login(&self, answer: &ClientAnswer) -> Result<LoginResponse, DomainError> {
        self.post("/users/login", answer).await
    }

    async fn post<B, D>(&self, path: &str, body: &B) -> Result<D, DomainError>
    where
        B: Serialize,
        D: DTO,
    {
        let url = self.host.e(path);
        let resp: Response<D> = Request::post(&url.0)
            .json(body)
            .map_err(|e| DomainError::Serialize(e.to_string()))?
            .send()
            .await
            .map_err(|e| DomainError::HttpConnectionFailed(e.to_string()))?
            .json()
            .await
            .map_err(|e| DomainError::Serialize(e.to_string()))?;
        resp.get()
    }
}
