use crate::application::ports::{clients, repository};

pub trait AgentRepos:
    repository::container::HasAgentCommand + repository::container::HasAgentCommand
{
}

impl<T: repository::container::HasAgentCommand> AgentRepos for T {}

pub trait AgentClients:
    clients::container::HasDiffusion
    + clients::container::HasStorage
    + clients::container::HasEngines
    + clients::container::HasNotification
{
}

impl<
        T: clients::container::HasDiffusion
            + clients::container::HasStorage
            + clients::container::HasEngines
            + clients::container::HasNotification,
    > AgentClients for T
{
}
