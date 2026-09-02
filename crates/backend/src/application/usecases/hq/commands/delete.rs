use crate::application::ports::{
    repository::{agent_command::AgentCommandRepository, container::HasAgentCommand},
    usecase::UsecaseAPI,
};
use domain::errors::DomainError;
use dto::resources::commands::{DeletePayload, GetListResponse};
use pkg::macros::trait_repos;

trait_repos!(DeleteCommandRepos, HasAgentCommand);

// #[route(delete, path = "/")]
pub struct DeleteCommand<'a, R: DeleteCommandRepos> {
    repos: &'a R,
    payload: DeletePayload,
}

impl<'a, R: DeleteCommandRepos> DeleteCommand<'a, R> {
    pub fn new(repos: &'a R, payload: DeletePayload) -> Self {
        Self { repos, payload }
    }
}

impl<R: DeleteCommandRepos> UsecaseAPI<GetListResponse> for DeleteCommand<'_, R> {
    async fn exec(&self) -> Result<GetListResponse, domain::errors::DomainError> {
        let repo = self.repos.agent_command();
        let _: Result<(), DomainError> = repo
            .delete(&self.payload.action_id)
            .await
            .map_err(|x| x.into());
        Ok(GetListResponse { commands: vec![] })
    }
}
