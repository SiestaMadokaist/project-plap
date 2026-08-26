use reqwest::Client;
use serde::Serialize;

use crate::application::ports::clients::notification::NotificationClient;
use crate::infras::http_error::reqwest_error;
use domain::errors::DomainError;
use pkg::types::strings::URL;

#[derive(Serialize)]
struct WebhookPayload<'a> {
    username: &'a str,
    content: &'a str,
}

pub struct Discord {
    client: Client,
    username: String,
    webhook_url: URL,
}

impl Discord {
    pub fn new(username: String, webhook_url: URL) -> Self {
        Self {
            username,
            client: Client::new(),
            webhook_url,
        }
    }
}

impl NotificationClient for Discord {
    async fn notify(&self, info: &str) -> Result<(), DomainError> {
        let payload = WebhookPayload {
            username: &self.username,
            content: info,
        };
        self.client
            .post(&self.webhook_url.0)
            .json(&payload)
            .send()
            .await
            .map_err(reqwest_error)?;
        Ok(())
    }
}
