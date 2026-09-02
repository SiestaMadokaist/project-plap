use domain::{
    errors::DomainError,
    storage::{StoragePath, StoragePrefix},
};
use dto::{resources::list::ListResponse, response::Response};
use gloo_net::http::Request;
use pkg::{auth::claims::JWT, types::strings::URL};

pub struct PlapApi {
    auth: JWT,
    host: URL,
}

impl PlapApi {
    pub fn new(auth: JWT, host: URL) -> Self {
        Self { auth, host }
    }

    pub async fn list_models(
        &self,
        prefix: StoragePrefix,
    ) -> Result<ListResponse<StoragePath>, DomainError> {
        let payload = dto::resources::models::GetListPayload { prefix };
        let url = self.host.e("/models/list");
        let builder = Request::post(&url.0)
            .json(&payload)
            .map_err(|x| DomainError::Serialize(x.to_string()))?;
        let resp = self.send::<ListResponse<StoragePath>>(builder).await?;
        resp.get()
    }

    async fn send<D: dto::response::DTO>(&self, req: Request) -> Result<Response<D>, DomainError> {
        req.headers().set("Authorization", &self.auth.0);
        let resp: Response<D> = req
            .send()
            .await
            .map_err(|x| DomainError::HttpConnectionFailed(x.to_string()))?
            .json()
            .await
            .map_err(|x| DomainError::Serialize(x.to_string()))?;
        Ok(resp)
        // resp.and_then(|x| x.get())
    }
}
