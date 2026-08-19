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
        let name = &mv.name;
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
    #[cfg(feature = "datatransfer")]
    use std::path::PathBuf;

    use super::*;
    use crate::{
        application::ports::clients::inference_model_provider::InferenceModelProvider,
        domain::storage::{StorageBucket, StoragePrefix},
        pkg::{
            civitai::typing::ModelCategory,
            id::InferenceModelId,
            types::unit::{Index0, INDEX_ZERO},
        },
    };

    struct MockStorage {}

    #[allow(async_fn_in_trait)]
    impl StorageClient for MockStorage {
        fn provider_name() -> String {
            "mock".into()
        }
        fn bucket(&self) -> StorageBucket {
            StorageBucket("mock-bucket".into())
        }
        async fn read(&self, path: &StoragePath) -> Result<String, DomainError> {
            todo!()
        }
        async fn write(&self, path: &StoragePath, data: &Vec<u8>) -> Result<(), DomainError> {
            todo!()
        }
        fn public_url(&self, path: &StoragePath) -> String {
            todo!()
        }
        async fn ls(&self, prefix: &StoragePrefix) -> Vec<String> {
            todo!()
        }
        #[cfg(feature = "datatransfer")]
        async fn upload(&self, local: &PathBuf, remote: &StoragePath) -> Result<(), DomainError> {
            todo!()
        }
        #[cfg(feature = "datatransfer")]
        async fn download(&self, remote: &StoragePath, local: &PathBuf) -> Result<(), DomainError> {
            todo!()
        }
        #[cfg(feature = "datatransfer")]
        fn abs_path(&self, local: &PathBuf) -> PathBuf {
            todo!()
        }
    }
    struct MockInferenceModelProvider {}

    #[async_trait::async_trait(?Send)]
    impl InferenceModelProvider for MockInferenceModelProvider {
        async fn get_detail(&self, id: &InferenceModelId) -> anyhow::Result<ModelVersionDTO> {
            todo!()
        }
        fn abs_path(&self, id: &InferenceModelId, category: &ModelCategory, name: &str) -> PathBuf {
            todo!()
        }
        #[cfg(feature = "datatransfer")]
        async fn download(&self, id: &InferenceModelId, dst: &PathBuf) -> anyhow::Result<()> {
            todo!()
        }
    }

    struct MockContainer {
        storage: MockStorage,
        civitai: MockInferenceModelProvider,
    }

    impl MockContainer {
        fn rc() -> Rc<Self> {
            let container = Self {
                storage: MockStorage {},
                civitai: MockInferenceModelProvider {},
            };
            Rc::new(container)
        }
    }

    impl HasModelStorage for MockContainer {
        type ModelStorage = MockStorage;
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
        let clients = MockContainer::rc();
        let arg_json =
            serde_json::from_str(r"{}").map_err(|x| DomainError::Serialize(x.to_string()))?;
        let args: NetworkArgs =
            serde_json::from_value(arg_json).map_err(|x| DomainError::Serialize(x.to_string()))?;
        let handler = HandleNetwork::new(clients, &args, Progression::new(Index0(1), INDEX_ZERO));
        let result = handler.exec().await;
        Ok(())
    }
}
