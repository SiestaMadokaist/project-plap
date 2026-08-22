#[cfg(feature = "datatransfer")]
use crate::domain::commands::command::Progression;
use crate::{
    application::ports::clients::container::{HasInferenceModelProvider, HasModelStorage},
    application::ports::clients::storage::StorageClient,
    domain::storage::StoragePath,
    domain::{
        commands::network::{ModelDst, ModelSrc, NetworkArgs},
        errors::DomainError,
    },
    pkg::civitai::{self, dto::model_version::ModelVersionDTO},
    pkg::macros::trait_clients,
};
use std::rc::Rc;

trait_clients!(
    HandleNetworkClients,
    HasModelStorage,
    HasInferenceModelProvider
);

#[cfg(feature = "datatransfer")]
pub struct HandleNetwork<'a, C: HandleNetworkClients> {
    clients: Rc<C>,
    args: &'a NetworkArgs,
    // c: StorageClient
    // network command always complete in 1 execution?
    progress: Progression,
}

#[cfg(feature = "datatransfer")]
impl<'a, C: HandleNetworkClients> HandleNetwork<'a, C> {
    pub fn new(clients: Rc<C>, args: &'a NetworkArgs, progress: Progression) -> Self {
        Self {
            clients,
            args,
            progress,
        }
    }

    fn remote_path(mv: &ModelVersionDTO, ext: &str) -> StoragePath {
        let category = mv.category();
        let id: &civitai::typing::VersionId = &mv.id;
        let name = &mv.name();
        let s = format!("{category}/{id}/{name}{ext}");
        StoragePath(s)
    }

    async fn handle_network(&self) -> Result<Progression, DomainError> {
        let arg = self.args;
        let mut progress = self.progress.clone();
        let result: Result<Progression, DomainError> = match &arg.src {
            ModelSrc::S3(s) => match &arg.dst {
                ModelDst::Local(d) => {
                    let storage = self.clients.model_storage();
                    let abs_path = storage.abs_path(&d.path);
                    storage.download(&s.path, &abs_path).await?;
                    if d.forward {
                        let fwd = d.path.to_str().unwrap_or_default();
                        if fwd == "" {
                            let msg = "local path must be defined first";
                            let err = DomainError::Prerequisite(msg.into());
                            Err(err.into())
                        } else {
                            let abs_path = storage.abs_path(&d.path);
                            storage.upload(&abs_path, &StoragePath(fwd.into())).await?;
                            progress.increment();
                            Ok(progress)
                        }
                    } else {
                        progress.increment();
                        Ok(progress)
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
                    let api = self.clients.inference_model_provider();
                    let mv = api
                        .get_detail(id)
                        .await
                        .map_err(|x| DomainError::ApiError(x.to_string()))?;
                    let path = api.abs_path(
                        &mv.id,
                        &mv.category(),
                        &format!("{}.safetensors", &mv.name()),
                    );
                    api.download(id, &path)
                        .await
                        .map_err(|x| DomainError::HttpConnectionFailed(x.to_string()))?;
                    if args.forward {
                        let storage = self.clients.model_storage();
                        let model_path = Self::remote_path(&mv, ".safetensors");
                        let info_path = Self::remote_path(&mv, ".civitai.json");
                        storage.upload(&path, &model_path).await?;
                        let info = serde_json::to_value(mv)?.to_string();
                        storage
                            .write(&info_path, &info.to_string().into_bytes())
                            .await?;
                    }
                    progress.increment();
                    Ok(progress)
                }
            },
        };
        result
    }

    pub async fn exec(&self) -> Result<Progression, DomainError> {
        self.handle_network().await
    }
}

#[cfg(all(test, feature = "datatransfer"))]
mod tests {

    use super::*;
    use crate::{
        application::ports::clients::{
            inference_model_provider::{InferenceModelProvider, MockInferenceModelProvider},
            storage::MockStorageClient,
        },
        pkg::types::unit::{Index0, INDEX_ZERO},
    };

    struct MockContainer {
        storage: MockStorageClient,
        civitai: MockInferenceModelProvider,
    }

    impl MockContainer {
        fn rc() -> Rc<Self> {
            let buffer =
                std::fs::read("./samples/inputs/jsons/infras/civitai/resp.version.checkpoint.json")
                    .expect("cannot find resp.version.checkpoint.json");
            let mv: ModelVersionDTO =
                serde_json::from_slice(&buffer).expect("cannot deserialize fixture");

            let mut civitai = MockInferenceModelProvider::new();
            civitai
                .expect_get_detail()
                .returning(move |_| Ok(mv.clone()));
            civitai
                .expect_abs_path()
                .returning(|_, _, _| "/root/path/imp/to/x".into());
            #[cfg(feature = "datatransfer")]
            civitai.expect_download().returning(|_, _| Ok(()));
            let storage = MockStorageClient::new();
            let container = Self { storage, civitai };
            Rc::new(container)
        }
    }

    impl HasModelStorage for MockContainer {
        type ModelStorage = MockStorageClient;
        fn model_storage(&self) -> &Self::ModelStorage {
            &self.storage
        }
    }

    impl HasInferenceModelProvider for MockContainer {
        fn inference_model_provider(&self) -> &dyn InferenceModelProvider {
            &self.civitai
        }
    }

    #[tokio::test]
    async fn s32localhost() -> Result<(), DomainError> {
        let buffer = std::fs::read("./samples/inputs/jsons/domain/commands/network2.json")
            .expect("cannot find network2.json");
        let args: NetworkArgs = serde_json::from_slice(&buffer)
            .expect("cannot deserialize buffer to InferenceConfig<String>");

        let clients = MockContainer::rc();
        let handler = HandleNetwork::new(clients, &args, Progression::new(Index0(1), INDEX_ZERO));
        let updated = handler.exec().await?;
        assert!(updated.is_done());
        Ok(())
    }
}
