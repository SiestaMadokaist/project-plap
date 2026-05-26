use reqwest::Client;
use serde::Serialize;

use crate::{
    application::ports::clients::notification::NotificationClient, domain::errors::DomainError,
};

#[derive(Serialize)]
struct WebhookPayload<'a> {
    username: &'a str,
    content: &'a str,
}

pub struct Discord {
    client: Client,
    webhook_url: String,
}

impl Discord {
    pub fn new(webhook_url: String) -> Self {
        Self {
            client: Client::new(),
            webhook_url,
        }
    }
}

impl NotificationClient for Discord {
    async fn notify(&self, info: &str) -> Result<(), DomainError> {
        let payload = WebhookPayload {
            username: "zero-translation",
            content: info,
        };
        self.client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await?;
        Ok(())
    }
}
