use std::path::PathBuf;

use domain::{
    commands::{
        command::{ActionId, CommandDomain},
        network::{LocalArgs, ModelDst, ModelSrc, NetworkArgs, S3Args},
    },
    errors::DomainError,
    storage::{StorageBucket, StoragePath, StoragePrefix},
};
use dto::{
    resources::{commands::CpPayload, models},
    response::Response,
};
use gloo_net::http::Request;
use pkg::{auth::claims::JWT, types::strings::URL};

#[derive(Clone)]
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
        recursive: bool,
    ) -> Result<models::GetListResponse, DomainError> {
        let payload = dto::resources::models::GetListPayload { prefix, recursive };
        let url = self.host.e("/models/list");
        let builder = Request::post(&url.0)
            .json(&payload)
            .map_err(|x| DomainError::Serialize(x.to_string()))?;
        let resp = self.send::<models::GetListResponse>(builder).await?;
        resp.get()
    }

    /// Queue an s3 -> localhost copy of `src` into the same path with the leading
    /// `comfyui/` segment rewritten to `models/`. Returns the queued command.
    pub async fn cp_model(
        &self,
        bucket: StorageBucket,
        src: StoragePath,
    ) -> Result<CommandDomain, DomainError> {
        let dst = match src.0.strip_prefix("comfyui/") {
            Some(rest) => format!("models/{rest}"),
            None => {
                return Err(DomainError::Prerequisite(format!(
                    "expected a `comfyui/` path, got `{}`",
                    src.0
                )))
            }
        };

        let now = js_sys::Date::now();
        let payload = CpPayload {
            action_id: ActionId(format!("{}", now as i64)),
            priority: now as u64,
            args: NetworkArgs::new(
                ModelSrc::S3(S3Args { bucket, path: src }),
                ModelDst::Local(LocalArgs {
                    forward: false,
                    path: PathBuf::from(dst),
                }),
            ),
        };

        let url = self.host.e("/agents/command/cp");
        let builder = Request::post(&url.0)
            .json(&payload)
            .map_err(|x| DomainError::Serialize(x.to_string()))?;
        let resp = self
            .send::<dto::resources::commands::CpResponse>(builder)
            .await?
            .get()?;
        Ok(resp.command)
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
    }
}
