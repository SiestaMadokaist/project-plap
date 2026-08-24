use crate::{
    application::ports::clients::translator::TranslatorClient, domain::errors::DomainError,
};
use async_openai::{
    config::OpenAIConfig,
    error::OpenAIError,
    types::{ChatCompletionRequestMessage, CreateChatCompletionRequest, Model},
    Client,
};
use tokio::sync::OnceCell;
const SYSTEM_PROMPT: &str = include_str!("../../../prompts/translate.txt");

pub struct ChatGPT {
    client: Client<OpenAIConfig>,
    model_name: String,
    model_cache: OnceCell<Model>,
}

impl ChatGPT {
    pub fn new(c: Client<OpenAIConfig>, model: &str) -> Self {
        ChatGPT {
            client: c,
            model_name: String::from(model),
            model_cache: OnceCell::new(),
        }
    }

    pub async fn model(&self) -> Result<&Model, DomainError> {
        let init = self
            .model_cache
            .get_or_try_init(async || {
                let retrieved = self.client.models().retrieve(&self.model_name).await;
                retrieved
            })
            .await?;
        Ok(init)
    }

    fn system_prompt(&self) -> ChatCompletionRequestMessage {
        ChatCompletionRequestMessage::System(SYSTEM_PROMPT.into())
    }

    async fn translate_partial(&self, text: &str) -> Result<String, DomainError> {
        let model = self.model().await?;
        let system_prompt = self.system_prompt();
        let user_prompt = ChatCompletionRequestMessage::User(text.into());
        let chat = self.client.chat();
        let prompts = vec![system_prompt, user_prompt];
        let requests = CreateChatCompletionRequest {
            messages: prompts,
            model: model.id.clone(),
            ..Default::default()
        };
        let resp = chat.create(requests);
        let data = resp.await?;
        if data.choices.is_empty() {
            return Err(DomainError::EmptyResponse);
        }
        let response = &data.choices[0].message.content;
        match response {
            Some(s) => Ok(s.clone()),
            None => Err(DomainError::MissingContent),
        }
    }
}

const PARTITION_COUNT: usize = 3;
impl TranslatorClient for ChatGPT {
    async fn translate(&self, text: &str) -> Result<String, DomainError> {
        let paragraphs = text.split("\n");
        let p_count = paragraphs.clone().count();
        if p_count < PARTITION_COUNT {
            let result = self.translate_partial(text).await?;
            return Ok(result);
        }
        let partition_size = (p_count / PARTITION_COUNT) + 1; // ceil, or whatever
        let mut groups: Vec<Vec<&str>> = vec![vec![]; PARTITION_COUNT];
        for (i, p) in paragraphs.enumerate() {
            let group = &mut groups[i / partition_size];
            group.push(p);
        }
        let grouped_paragraphs: Vec<String> = groups.iter().map(|x| x.join("\n")).collect();
        let mut translations: Vec<String> = vec![];
        for g in grouped_paragraphs {
            let translated = self.translate_partial(&g).await?;
            translations.push(translated);
        }
        let full_translation = translations.join("\n");
        Ok(full_translation)
    }
}

impl From<OpenAIError> for DomainError {
    fn from(e: OpenAIError) -> Self {
        match e {
            OpenAIError::ApiError(ref err) if err.code == Some("rate_limit_exceeded".into()) => {
                DomainError::RateLimited
            }
            OpenAIError::ApiError(ref err) => DomainError::ApiError(err.message.clone()),
            _ => DomainError::ApiError(e.to_string()),
        }
    }
}
