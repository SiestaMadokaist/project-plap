use crate::{infras::civitai, pkg::types::strings::URL};

pub struct CivitaiAPI {
    host: URL,
    api_key: String,
    client: reqwest::Client,
}

impl CivitaiAPI {
    pub fn new(host: URL, api_key: String) -> Self {
        let client = reqwest::Client::new();
        Self {
            host,
            api_key,
            client,
        }
    }

    async fn send(&self, b: reqwest::RequestBuilder) -> anyhow::Result<reqwest::Response> {
        let authorization = format!("Bearer {}", self.api_key);
        let res = b.header("Authorization", authorization).send().await?;
        Ok(res)
    }

    pub async fn model_detail(&self, id: civitai::id::ModelId) -> anyhow::Result<()> {
        let url = self.host.e("/api/v1/model").e(&id.to_string());
        let req = self.client.get(url.to_string());
        let resp = self.send(req).await?;
        let data = resp.json::<civitai::id::Todo>().await;
        todo!();
    }

    pub async fn version_detail(&self, id: civitai::id::VersionId) -> anyhow::Result<()> {
        let url = self.host.e("/api/v1/version").e(&id.to_string());
        let req = self.client.get(url.to_string());
        let resp = self.send(req).await?;
        let data = resp.json::<civitai::id::Todo>().await;
        todo!();
    }

    // @todo: Pass Stream Pipe here??
    pub async fn download(&self, id: civitai::id::VersionId) -> anyhow::Result<()> {
        let url = self.host.e("/api/download/models").e(&id.to_string());
        let req = self.client.get(url.to_string());
        let resp = self.send(req).await?;
        let data = resp.json::<civitai::id::Todo>().await;
        todo!();
    }
}
