use crate::application::ports::clients::translator::{TranslateError, TranslatorClient};
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

    pub async fn model(&self) -> Result<&Model, OpenAIError> {
        let init = self
            .model_cache
            .get_or_try_init(async || {
                let retrieved = self.client.models().retrieve(&self.model_name).await;
                return retrieved;
            })
            .await;
        return init;
    }

    fn system_prompt(&self) -> ChatCompletionRequestMessage {
        return ChatCompletionRequestMessage::System(SYSTEM_PROMPT.into());
    }
}

impl TranslatorClient for ChatGPT {
    async fn translate(&self, text: &str) -> Result<String, TranslateError> {
        let model = self.model().await?;
        let system_prompt = self.system_prompt();
        let user_prompt = ChatCompletionRequestMessage::User(text.into());
        let chat = self.client.chat();
        let prompts = vec![system_prompt, user_prompt];
        let mut requests: CreateChatCompletionRequest = CreateChatCompletionRequest::default();
        requests.messages = prompts;
        requests.model = model.id.clone();
        let resp = chat.create(requests);
        let data = resp.await?;
        if data.choices.len() == 0 {
            return Err(TranslateError::EmptyResponse);
        }
        let response = &data.choices[0].message.content;
        return match response {
            Some(s) => Ok(s.clone()),
            None => Err(TranslateError::EmptyResponse),
        };
    }
}

impl From<OpenAIError> for TranslateError {
    fn from(e: OpenAIError) -> Self {
        match e {
            OpenAIError::ApiError(ref err) if err.code == Some("rate_limit_exceeded".into()) => {
                TranslateError::RateLimited
            }
            _ => TranslateError::ServiceUnavailable("ChatGPT".into()),
        }
    }
}
