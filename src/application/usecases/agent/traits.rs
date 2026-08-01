use crate::application::ports::{clients::ctrait, repository::rtrait};

pub trait AgentRepos: rtrait::HasUser {}
pub trait AgentClients: ctrait::HasDiffusion {}
