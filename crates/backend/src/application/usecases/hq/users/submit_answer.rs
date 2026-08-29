use std::rc::Rc;

use crate::application::ports::{
    clients::{authorizer::Authorizer, container::HasAuthValidator},
    repository::{container::HasUser, user::UserRepository},
    usecase::UsecaseAPI,
};
use domain::errors::DomainError;
use dto::resources::login::{ClientAnswer, LoginResponse};
use pkg::macros::{trait_clients, trait_repos};

trait_clients!(ISubmitAnswerClients, HasAuthValidator);
trait_repos!(ISubmitAnswerRepos, HasUser);

/// Step 2 of login: exchange a signed `ClientAnswer` for a session token. Challenge
/// authenticity, wallet recovery and expiry are checked in `Authorizer::answer`; the
/// repo `login` then stamps `last_login` (and enforces activation + a monotonic `iat`).
pub struct SubmitAnswer<C: ISubmitAnswerClients, R: ISubmitAnswerRepos> {
    clients: Rc<C>,
    repos: Rc<R>,
    payload: ClientAnswer,
}

impl<C: ISubmitAnswerClients, R: ISubmitAnswerRepos> SubmitAnswer<C, R> {
    pub fn new(clients: Rc<C>, repos: Rc<R>, payload: ClientAnswer) -> Self {
        Self {
            clients,
            repos,
            payload,
        }
    }
}

impl<C: ISubmitAnswerClients, R: ISubmitAnswerRepos> UsecaseAPI<LoginResponse>
    for SubmitAnswer<C, R>
{
    async fn exec(&self) -> Result<LoginResponse, DomainError> {
        let token = self
            .clients
            .authorizer()
            .answer(self.payload.clone())
            .await?;

        self.repos
            .user()
            .login(
                self.payload.challenge.address(),
                self.payload.challenge.iat(),
            )
            .await
            .map_err(|e| DomainError::NotAllowed(e.to_string()))?;

        Ok(LoginResponse { token })
    }
}
