use crate::application::ports::{clients, repository};

pub trait AgentRepos: repository::container::HasAgentCommand {}
pub trait AgentClients: clients::container::HasDiffusion {}
