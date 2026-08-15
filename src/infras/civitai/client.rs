use std::path::PathBuf;

use tokio::io::AsyncWriteExt;

use crate::{
    infras::civitai::{
        self,
        dto::{model_detail::ModelDetailDTO, model_version::ModelVersionDTO},
        typing::{self, VersionId},
    },
    pkg::types::strings::URL,
};

pub struct CivitaiAPI {
    host: URL,
    api_key: String,
    client: reqwest::Client,
    workdir: String,
}

impl CivitaiAPI {
    pub fn new(host: URL, api_key: String, workdir: String) -> Self {
        let client = reqwest::Client::new();
        Self {
            host,
            api_key,
            client,
            workdir,
        }
    }

    pub fn abs_path(&self, id: &VersionId, t: &typing::ModelCategory, name: &str) -> PathBuf {
        let dst = format!("{}/models/{}/{}_{}", self.workdir, t, id, name);
        PathBuf::from(dst)
    }

    async fn send(&self, b: reqwest::RequestBuilder) -> anyhow::Result<reqwest::Response> {
        let authorization = format!("Bearer {}", self.api_key);
        let res = b.header("Authorization", authorization).send().await?;
        Ok(res)
    }

    pub async fn model_detail(
        &self,
        id: &civitai::typing::ModelId,
    ) -> anyhow::Result<ModelDetailDTO> {
        let url = self.host.e("/api/v1/model").e(&id.to_string());
        let req = self.client.get(url.to_string());
        let resp = self.send(req).await?;
        let data = resp
            .json::<civitai::dto::model_detail::ModelDetailDTO>()
            .await?;
        Ok(data)
    }

    pub async fn version_detail(
        &self,
        id: &civitai::typing::VersionId,
    ) -> anyhow::Result<ModelVersionDTO> {
        let url = self.host.e("/api/v1/version").e(&id.to_string());
        let req = self.client.get(url.to_string());
        let resp = self.send(req).await?;
        let data = resp
            .json::<civitai::dto::model_version::ModelVersionDTO>()
            .await?;
        Ok(data)
    }

    #[cfg(feature = "datatransfer")]
    pub async fn download(
        &self,
        id: &civitai::typing::VersionId,
        dst: &PathBuf,
    ) -> anyhow::Result<()> {
        let url = self.host.e("/api/download/models/").e(&id.to_string());
        let req = self.client.get(url.to_string());
        let mut resp = self.send(req).await?.error_for_status()?;
        if let Some(parent) = dst.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let mut file = tokio::fs::File::create(&dst).await?;
        while let Some(chunk) = resp.chunk().await? {
            file.write_all(&chunk).await?;
        }
        Ok(())
    }
}
