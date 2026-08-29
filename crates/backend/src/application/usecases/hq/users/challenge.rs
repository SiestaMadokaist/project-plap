use std::rc::Rc;

use crate::application::ports::{
    clients::{authorizer::Authorizer, container::HasAuthValidator},
    repository::{container::HasUser, user::UserRepository},
    usecase::UsecaseAPI,
};
use domain::errors::DomainError;
use dto::resources::login::{ReqChallenge, ServerChallenge};
use pkg::macros::{trait_clients, trait_repos};

trait_clients!(IChallengeClients, HasAuthValidator);
trait_repos!(IChallengeRepos, HasUser);

/// Step 1 of login: mint a signed `ServerChallenge` for the wallet named in the request.
/// The address must already belong to a known user; timing checks and the server
/// signature live in `Authorizer::challenge`.
pub struct GetChallenge<C: IChallengeClients, R: IChallengeRepos> {
    clients: Rc<C>,
    repos: Rc<R>,
    payload: ReqChallenge,
}

impl<C: IChallengeClients, R: IChallengeRepos> GetChallenge<C, R> {
    pub fn new(clients: Rc<C>, repos: Rc<R>, payload: ReqChallenge) -> Self {
        Self {
            clients,
            repos,
            payload,
        }
    }
}

impl<C: IChallengeClients, R: IChallengeRepos> UsecaseAPI<ServerChallenge> for GetChallenge<C, R> {
    async fn exec(&self) -> Result<ServerChallenge, DomainError> {
        self.repos
            .user()
            .find(self.payload.address())
            .await
            .map_err(|e| DomainError::NotAllowed(e.to_string()))?;

        self.clients
            .authorizer()
            .challenge(self.payload.clone())
            .await
    }
}
