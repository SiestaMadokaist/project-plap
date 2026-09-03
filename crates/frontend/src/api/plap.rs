use std::path::PathBuf;

use domain::{
    commands::{
        command::{ActionId, CommandDomain, CommandStage},
        network::{LocalArgs, ModelDst, ModelSrc, NetworkArgs, S3Args},
    },
    errors::DomainError,
    storage::{StorageBucket, StoragePath, StoragePrefix},
};
use dto::{
    resources::{commands, commands::CpModelPayload, models},
    response::Response,
};
use gloo_net::http::Request;
use pkg::{auth::claims::JWT, id::InferenceModelId, types::strings::URL};

#[derive(Clone)]
pub struct PlapApi {
    auth: JWT,
    host: URL,
    /// bucket the `comfyui/…` model tree is listed and copied from
    model_bucket: StorageBucket,
    /// bucket for generated input/output artifacts — not consumed yet, carried
    /// so callers don't have to thread it in later
    #[allow(dead_code)]
    io_bucket: StorageBucket,
}

impl PlapApi {
    pub fn new(
        auth: JWT,
        host: URL,
        model_bucket: StorageBucket,
        io_bucket: StorageBucket,
    ) -> Self {
        Self {
            auth,
            host,
            model_bucket,
            io_bucket,
        }
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
    pub async fn cp_model(&self, src: StoragePath) -> Result<CommandDomain, DomainError> {
        let dst = match src.0.strip_prefix("comfyui/") {
            Some(rest) => format!("models/{rest}"),
            None => {
                return Err(DomainError::Prerequisite(format!(
                    "expected a `comfyui/` path, got `{}`",
                    src.0
                )))
            }
        };

        // deterministic id: re-queuing the same object overwrites its command
        // rather than piling up duplicates.
        let action_id = ActionId(format!("Network-{}", src.0));
        let payload = CpModelPayload {
            action_id,
            priority: js_sys::Date::now() as u64,
            args: NetworkArgs::new(
                ModelSrc::S3(S3Args {
                    bucket: self.model_bucket.clone(),
                    path: src,
                }),
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
            .send::<dto::resources::commands::CpModelResponse>(builder)
            .await?
            .get()?;
        Ok(resp.command)
    }

    /// Queue the fixed `comfyui/bootstraps/ -> models/` prefix copy — the comfyui
    /// runtime configs the agent needs alongside the models. Unlike [`Self::cp_model`]
    /// the destination stays flat at `models/` (not `models/bootstraps/`).
    pub async fn cp_bootstraps(&self) -> Result<CommandDomain, DomainError> {
        let payload = CpModelPayload {
            action_id: ActionId("Network-comfyui/bootstraps/".into()),
            priority: js_sys::Date::now() as u64,
            args: NetworkArgs::new(
                ModelSrc::S3(S3Args {
                    bucket: self.model_bucket.clone(),
                    path: StoragePath("bootstraps/".into()),
                }),
                ModelDst::Local(LocalArgs {
                    forward: false,
                    path: PathBuf::from("models/"),
                }),
            ),
        };

        let url = self.host.e("/agents/command/cp");
        let builder = Request::post(&url.0)
            .json(&payload)
            .map_err(|x| DomainError::Serialize(x.to_string()))?;
        let resp = self
            .send::<dto::resources::commands::CpModelResponse>(builder)
            .await?
            .get()?;
        Ok(resp.command)
    }

    /// Queue a civitai -> localhost download of model version `id`. `forward: true`
    /// tells the agent to push the file on to its own storage after fetching, so
    /// the destination path is a placeholder.
    pub async fn cp_civitai(&self, id: u32) -> Result<CommandDomain, DomainError> {
        let payload = CpModelPayload {
            action_id: ActionId(format!("Network-civitai-{id}")),
            priority: js_sys::Date::now() as u64,
            args: NetworkArgs::new(
                ModelSrc::Civitai(InferenceModelId(id)),
                ModelDst::Local(LocalArgs {
                    forward: true,
                    path: PathBuf::from("_"),
                }),
            ),
        };

        let url = self.host.e("/agents/command/cp");
        let builder = Request::post(&url.0)
            .json(&payload)
            .map_err(|x| DomainError::Serialize(x.to_string()))?;
        let resp = self
            .send::<dto::resources::commands::CpModelResponse>(builder)
            .await?
            .get()?;
        Ok(resp.command)
    }

    /// Drop one queued command by its `action_id`.
    pub async fn delete_command(&self, action_id: ActionId) -> Result<(), DomainError> {
        let payload = commands::DeletePayload { action_id };
        let url = self.host.e("/agents/command/delete");
        let builder = Request::post(&url.0)
            .json(&payload)
            .map_err(|x| DomainError::Serialize(x.to_string()))?;
        self.send::<commands::GetListResponse>(builder)
            .await?
            .get()?;
        Ok(())
    }

    /// List agent commands currently in the queue (stage `in_progress`).
    pub async fn list_taskqueue(&self) -> Result<commands::GetListResponse, DomainError> {
        let payload = commands::GetListPayload {
            stage: CommandStage::InProgress,
            limit: 100,
        };
        let url = self.host.e("/agents/command/list");
        let builder = Request::post(&url.0)
            .json(&payload)
            .map_err(|x| DomainError::Serialize(x.to_string()))?;
        self.send::<commands::GetListResponse>(builder).await?.get()
    }

    /// Fetch a preview for one collapsed entry: a presigned url for its image
    /// sibling and/or the raw text of its json sibling. Either may be absent.
    pub async fn preview(
        &self,
        image: Option<StoragePath>,
        json: Option<StoragePath>,
    ) -> Result<models::PreviewResponse, DomainError> {
        let payload = models::PreviewPayload { image, json };
        let url = self.host.e("/models/preview");
        let builder = Request::post(&url.0)
            .json(&payload)
            .map_err(|x| DomainError::Serialize(x.to_string()))?;
        self.send::<models::PreviewResponse>(builder).await?.get()
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
