use std::rc::Rc;

use crate::{
    application::ports::repository::{
        agent_command::AgentCommandRepository, container::HasAgentCommand,
    },
    domain::{
        commands::command::{CommandDomain, CommandStage},
        errors::DomainError,
    },
    pkg::macros::trait_repos,
};

trait_repos!(ListCommandRepos, HasAgentCommand);

pub struct Payload {
    stage: CommandStage,
    limit: i32,
}
pub struct ListCommand<R: ListCommandRepos> {
    repos: Rc<R>,
    payload: Payload,
}

impl<R: ListCommandRepos> ListCommand<R> {
    pub fn new(repos: Rc<R>, payload: Payload) -> Self {
        Self { repos, payload }
    }

    pub async fn exec(&self) -> Result<Vec<CommandDomain>, DomainError> {
        let command_repo = self.repos.agent_command();
        let payload = &self.payload;
        let in_progress = command_repo
            .by_stage(payload.stage, payload.limit)
            .await
            .map_err(|x| DomainError::Disconnected(x.to_string()))?;
        Ok(in_progress)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        application::{
            ports::repository::{
                agent_command::MockAgentCommandRepository, container::HasAgentCommand,
                error::RepositoryError,
            },
            usecases::hq::commands::list_command::{self, ListCommand},
        },
        domain::{commands::command::CommandStage, errors::DomainError},
    };
    use std::rc::Rc;

    struct MockRepos {
        agent_command: MockAgentCommandRepository,
    }

    impl MockRepos {
        pub fn rc(agent_command: MockAgentCommandRepository) -> Rc<Self> {
            let s = Self { agent_command };
            Rc::new(s)
        }
    }

    impl HasAgentCommand for MockRepos {
        type AgentCommand = MockAgentCommandRepository;
        fn agent_command(&self) -> &Self::AgentCommand {
            &self.agent_command
        }
    }

    #[tokio::test]
    async fn error_handling() -> Result<(), DomainError> {
        let payload = list_command::Payload {
            stage: CommandStage::InProgress,
            limit: -1,
        };
        let mut agent = MockAgentCommandRepository::new();
        agent
            .expect_by_stage()
            .returning(|_, _| Err(RepositoryError::Database("something went wrong".into())));
        let repos = MockRepos::rc(agent);
        let usecase = ListCommand::new(repos, payload);
        let result = usecase.exec().await;
        assert!(matches!(result, Err(DomainError::Disconnected(_))));
        Ok(())
    }
}
