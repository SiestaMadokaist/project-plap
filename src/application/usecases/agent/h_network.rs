#[cfg(feature = "datatransfer")]
use crate::{
    application::ports::clients::container::{HasCivitai, HasModelStorage},
    application::ports::clients::storage::StorageClient,
    domain::storage::StoragePath,
    domain::{
        commands::{
            command::CommandStage,
            network::{ModelDst, ModelSrc, NetworkArgs},
        },
        errors::DomainError,
    },
    infras::civitai::{self, dto::model_version::ModelVersionDTO},
    pkg::macros::trait_clients,
};
use std::rc::Rc;

trait_clients!(HandleNetworkClients, HasModelStorage, HasCivitai);

#[cfg(feature = "datatransfer")]
pub struct HandleNetwork<'a, C: HandleNetworkClients> {
    clients: Rc<C>,
    args: &'a NetworkArgs,
    // c: StorageClient
    // network command always complete in 1 execution?
    // progress: Progression
}

#[cfg(feature = "datatransfer")]
impl<'a, C: HandleNetworkClients> HandleNetwork<'a, C> {
    pub fn new(clients: Rc<C>, args: &'a NetworkArgs) -> Self {
        Self { clients, args }
    }

    fn remote_path(mv: &ModelVersionDTO, ext: &str) -> StoragePath {
        let category = mv.category();
        let id: &civitai::typing::VersionId = &mv.id;
        let name = &mv.name;
        let s = format!("{category}/{id}/{name}{ext}");
        StoragePath(s)
    }

    async fn handle_network(&self) -> Result<CommandStage, DomainError> {
        let arg = self.args;
        let result: Result<CommandStage, DomainError> = match &arg.src {
            ModelSrc::S3(s) => match &arg.dst {
                ModelDst::Local(d) => {
                    let storage = self.clients.model_storage();
                    storage.download(&s.path, &d.path).await?;
                    if d.forward {
                        let fwd = d.path.to_str().unwrap_or_default();
                        if fwd == "" {
                            let msg = "local path must be defined first";
                            let err = DomainError::Prerequisite(msg.into());
                            Err(err.into())
                        } else {
                            storage.upload(&d.path, &StoragePath(fwd.into())).await?;
                            Ok(CommandStage::Completed)
                        }
                    } else {
                        Ok(CommandStage::Completed)
                    }
                }
                ModelDst::S3(_) => {
                    let msg = "transfer between s3 is not supported";
                    let err = DomainError::NotAllowed(msg.into());
                    Err(err.into())
                }
            },
            ModelSrc::Civitai(id) => match &arg.dst {
                ModelDst::S3(_) => {
                    let msg = "external to s3 must use local with forward = true";
                    let err = DomainError::NotAllowed(msg.into());
                    Err(err.into())
                }
                ModelDst::Local(args) => {
                    let api = self.clients.civitai();
                    let mv = api
                        .version_detail(id)
                        .await
                        .map_err(|x| DomainError::ApiError(x.to_string()))?;
                    let path = api.abs_path(&mv.id, &mv.category(), &mv.name);
                    api.download(id, &path)
                        .await
                        .map_err(|x| DomainError::HttpConnectionFailed(x.to_string()))?;
                    if args.forward {
                        let storage = self.clients.model_storage();
                        let model_path = Self::remote_path(&mv, ".safetensors");
                        let info_path = Self::remote_path(&mv, ".json");
                        storage.upload(&path, &model_path).await?;
                        let info = serde_json::to_value(mv)?.to_string();
                        storage
                            .write(&info_path, &info.to_string().into_bytes())
                            .await?;
                    }
                    Ok(CommandStage::Completed)
                }
            },
        };
        result
    }

    pub async fn exec(&self) -> Result<CommandStage, DomainError> {
        self.handle_network().await
    }
}
