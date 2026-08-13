use crate::{
    application::ports::{clients, repository},
    pkg::macros::{trait_clients, trait_repos},
};

trait_repos!(
    AgentRepos,
    repository::container::HasAgentCommand,
    repository::container::HasHotReload,
    repository::container::HasPromptHistory
);

trait_clients!(
    AgentClients,
    clients::container::HasDiffusion,
    clients::container::HasStorage,
    clients::container::HasEngines,
    clients::container::HasNotification,
    clients::container::HasComputeAgent,
);
