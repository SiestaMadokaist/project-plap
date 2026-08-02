use crate::application::ports::repository::{
    agent_command::AgentCommandRepository, r#macro::impl_has, translation::TranslationRepository,
    user::UserRepository,
};

pub trait RepositoryContainer {
    type User: UserRepository;
    type Translation: TranslationRepository;
    type AgentCommand: AgentCommandRepository;
    fn user(&self) -> &Self::User;
    fn translation(&self) -> &Self::Translation;
    fn agent_command(&self) -> &Self::AgentCommand;
}

pub trait HasAgentCommand {
    type AgentCommand: AgentCommandRepository;
    fn agent_command(&self) -> &Self::AgentCommand;
}

impl_has!(
    HasAgentCommand,
    AgentCommand,
    agent_command,
    RepositoryContainer
);

pub trait HasUser {
    type User: UserRepository;
    fn user(&self) -> &Self::User;
}
impl_has!(HasUser, User, user, RepositoryContainer);

pub trait HasTranslation {
    type Translation: TranslationRepository;
    fn translation(&self) -> &Self::Translation;
}
impl_has!(
    HasTranslation,
    Translation,
    translation,
    RepositoryContainer
);

pub trait AllRepos: HasTranslation + HasUser + HasAgentCommand {}
