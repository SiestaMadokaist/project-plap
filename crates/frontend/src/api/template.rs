use domain::{
    errors::DomainError,
    storyline::{Storyline, StoryTemplateId},
};
use dto::{
    resources::templates::{
        ListStoryTemplate, ReadTemplatePayload, ReadTemplateResponse, WriteTemplatePayload,
    },
    response::{Placeholder, Response},
};
use gloo_net::http::Request;
use pkg::{auth::claims::JWT, types::strings::URL};

#[derive(Clone)]
pub struct TemplateApi {
    auth: JWT,
    host: URL,
}

impl TemplateApi {
    pub fn new(auth: JWT, host: URL) -> Self {
        Self { auth, host }
    }

    /// Ids of every template belonging to the caller.
    pub async fn list(&self) -> Result<Vec<String>, DomainError> {
        let url = self.host.e("/story/templates");
        let builder = Request::post(&url.0)
            .json(&serde_json::json!({}))
            .map_err(|x| DomainError::Serialize(x.to_string()))?;
        let resp = self.send::<ListStoryTemplate>(builder).await?.get()?;
        Ok(resp.list.into_iter().map(|id| id.0).collect())
    }

    /// Fetch one template.
    pub async fn read(&self, id: &str) -> Result<Storyline, DomainError> {
        let payload = ReadTemplatePayload(StoryTemplateId(id.to_string()));
        let url = self.host.e("/story/templates/read");
        let builder = Request::post(&url.0)
            .json(&payload)
            .map_err(|x| DomainError::Serialize(x.to_string()))?;
        let resp = self.send::<ReadTemplateResponse>(builder).await?.get()?;
        Ok(resp.0)
    }

    /// Write back an edited template — same `id` as [`Self::read`] returned,
    /// with `inferences` / `variables` changed as needed.
    pub async fn write(&self, storyline: Storyline) -> Result<(), DomainError> {
        let payload = WriteTemplatePayload(storyline);
        let url = self.host.e("/story/templates/create");
        let builder = Request::post(&url.0)
            .json(&payload)
            .map_err(|x| DomainError::Serialize(x.to_string()))?;
        self.send::<Placeholder>(builder).await?.get()?;
        Ok(())
    }

    async fn send<D: dto::response::DTO>(&self, req: Request) -> Result<Response<D>, DomainError> {
        req.headers().set("Authorization", &self.auth.0);
        req.send()
            .await
            .map_err(|x| DomainError::HttpConnectionFailed(x.to_string()))?
            .json()
            .await
            .map_err(|x| DomainError::Serialize(x.to_string()))
    }
}
