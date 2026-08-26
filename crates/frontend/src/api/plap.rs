use domain::{errors::DomainError, storage::StoragePrefix};
use dto::response::Response;
use gloo_net::http::Request;
use pkg::types::strings::{JWT, URL};

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
    ) -> Result<dto::resources::models::GetListResponse, DomainError> {
        let payload = dto::resources::models::GetListPayload { prefix };
        let url = self.host.e("/models/list");
        let builder = Request::post(&url.0)
            .json(&payload)
            .map_err(|x| DomainError::Serialize(x.to_string()))?;
        let resp = self
            .send::<dto::resources::models::GetListResponse>(builder)
            .await
            .map_err(|x| DomainError::HttpConnectionFailed(x.to_string()))?;
        Ok(resp)
    }

    async fn send<D: dto::response::DTO>(&self, req: Request) -> Result<D, DomainError> {
        req.headers().set("Authorization", &self.auth.0);
        let resp: Result<Response<D>, DomainError> = req
            .send()
            .await
            .map_err(|x| DomainError::HttpConnectionFailed(x.to_string()))?
            .json()
            .await
            .map_err(|x| DomainError::Serialize(x.to_string()))?;
        resp.and_then(|x| x.get())
    }
}
