use crate::application::ports::repository::{
    agent_command::AgentCommandRepository, hot_reload::HotReloadRepository,
    prompt_history::PromptHistoryRepository, story::StoryTemplateRepository,
    translation::TranslationRepository, user::UserRepository,
};

pub trait HasAgentCommand {
    type AgentCommand: AgentCommandRepository;
    fn agent_command(&self) -> &Self::AgentCommand;
}
pub trait HasUser {
    type User: UserRepository;
    fn user(&self) -> &Self::User;
}

pub trait HasTranslation {
    type Translation: TranslationRepository;
    fn translation(&self) -> &Self::Translation;
}

pub trait HasHotReload {
    type HotReload: HotReloadRepository;
    fn hotreload(&self) -> &Self::HotReload;
}

pub trait HasPromptHistory {
    type PromptHistory: PromptHistoryRepository;
    fn prompt(&self) -> &Self::PromptHistory;
}

pub trait HasStoryTemplate {
    type StoryTemplate: StoryTemplateRepository;
    fn story_template(&self) -> &Self::StoryTemplate;
}
