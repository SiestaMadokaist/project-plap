use domain::errors::DomainError;
use dto::resources::login::{ClientAnswer, ReqChallenge, ServerChallenge};
use pkg::auth::claims::{Claims, JWT};

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait(?Send)]
pub trait Authorizer {
    async fn challenge(&self, req: ReqChallenge) -> Result<ServerChallenge, DomainError>;
    async fn answer(&self, ans: ClientAnswer) -> Result<JWT, DomainError>;
    // for now we assume its always claims
    async fn validate(&self, jwt: JWT) -> Result<Claims, DomainError>;
}
