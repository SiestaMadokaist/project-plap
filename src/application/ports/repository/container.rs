use crate::application::ports::repository::{
    agent_command::AgentCommandRepository, hot_reload::HotReloadRepository, r#macro::impl_has,
    translation::TranslationRepository, user::UserRepository,
};

pub trait RepositoryContainer {
    type User: UserRepository;
    type Translation: TranslationRepository;
    type AgentCommand: AgentCommandRepository;
    type HotReload: HotReloadRepository;
    fn user(&self) -> &Self::User;
    fn translation(&self) -> &Self::Translation;
    fn agent_command(&self) -> &Self::AgentCommand;
    fn hotreload(&self) -> &Self::HotReload;
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

pub trait HasHotReload {
    type HotReload: HotReloadRepository;
    fn hotreload(&self) -> &Self::HotReload;
}

impl_has!(HasHotReload, HotReload, hotreload, RepositoryContainer);

pub trait AllRepos: HasTranslation + HasUser + HasAgentCommand + HasHotReload {}
