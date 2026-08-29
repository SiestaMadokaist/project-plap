#[cfg(feature = "datatransfer")]
use std::rc::Rc;

use crate::application::ports::clients::{
    container::{HasInferenceModelProvider, HasModelStorage},
    storage::StorageClient,
};
#[cfg(feature = "datatransfer")]
use domain::commands::command::Progression;
use domain::{
    commands::network::{ModelDst, ModelSrc, NetworkArgs},
    errors::DomainError,
    storage::StoragePath,
};
use pkg::{
    civitai::{self, dto::model_version::ModelVersionDTO},
    macros::trait_clients,
};

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
                        if fwd.is_empty() {
                            let msg = "local path must be defined first";
                            let err = DomainError::Prerequisite(msg.into());
                            Err(err)
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
                    Err(err)
                }
            },
            ModelSrc::Civitai(id) => match &arg.dst {
                ModelDst::S3(_) => {
                    let msg = "external to s3 must use local with forward = true";
                    let err = DomainError::NotAllowed(msg.into());
                    Err(err)
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
                        &format!("{}.safetensors", mv.name()),
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

    pub async fn exec(&self) -> Progression {
        let result = self.handle_network().await;
        match result {
            Ok(p) => p,
            Err(e) => {
                let args = serde_json::to_string_pretty(self.args)
                    .unwrap_or("failed to deserialize args".into());
                tracing::error!(
                    "error: ({}) performing handle network\n```{}```",
                    e.to_string(),
                    args
                );
                let mut updated = self.progress.clone();
                updated.fail();
                updated
            }
        }
    }
}

#[cfg(all(test, feature = "datatransfer"))]
mod tests {

    use std::{path::PathBuf, rc::Rc};

    use super::*;
    use crate::application::ports::clients::{
        inference_model_provider::{InferenceModelProvider, MockInferenceModelProvider},
        storage::MockStorageClient,
    };
    use pkg::types::unit::{Index0, INDEX_ZERO};

    struct MockContainer {
        storage: MockStorageClient,
        civitai: MockInferenceModelProvider,
    }

    fn civitai() -> MockInferenceModelProvider {
        let buffer = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../samples/inputs/jsons/infras/civitai/resp.version.checkpoint.json"
        ))
        .expect("cannot find resp.version.checkpoint.json");
        let mv: ModelVersionDTO =
            serde_json::from_slice(&buffer).expect("cannot deserialize fixture");

        let mut civitai = MockInferenceModelProvider::new();
        civitai
            .expect_get_detail()
            .returning(move |_| Ok(mv.clone()));
        // civitai.abs_path(id, category, name)
        civitai
            .expect_abs_path()
            .returning(|_, _, _| PathBuf::from("/root/path/imp/to/x"));
        #[cfg(feature = "datatransfer")]
        civitai.expect_download().returning(|_, _| Ok(()));
        civitai
    }

    fn storage() -> MockStorageClient {
        let storage = MockStorageClient::new();
        storage
    }

    impl MockContainer {
        fn new(storage: MockStorageClient, civitai: MockInferenceModelProvider) -> Rc<Self> {
            let s = Self { storage, civitai };
            Rc::new(s)
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

    fn args() -> NetworkArgs {
        let buffer = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../samples/inputs/jsons/domain/commands/network2.json"
        ))
        .expect("cannot find network2.json");
        serde_json::from_slice(&buffer)
            .expect("cannot deserialize buffer to InferenceConfig<String>")
    }

    #[tokio::test]
    async fn s32localhost() -> Result<(), DomainError> {
        let mut storage = tests::storage();
        let civitai = tests::civitai();
        storage.expect_upload().returning(|_, _| Ok(()));
        storage.expect_write().returning(|_, _| Ok(()));
        let clients = MockContainer::new(storage, civitai);
        let args = tests::args();
        let handler = HandleNetwork::new(clients, &args, Progression::new(Index0(1), INDEX_ZERO));
        let result = handler.exec().await;
        assert!(result.is_done());
        Ok(())
    }

    #[tokio::test]
    async fn can_handle_error() -> Result<(), DomainError> {
        let args = tests::args();
        let civitai = tests::civitai();
        let mut storage = MockStorageClient::new();
        storage.expect_upload().returning(|_, _| Ok(()));
        storage
            .expect_write()
            .returning(|_path, _data| Err(DomainError::Disconnected("service error".into())));
        let clients = MockContainer::new(storage, civitai);
        let handler = HandleNetwork::new(clients, &args, Progression::new(Index0(1), INDEX_ZERO));
        let result = handler.exec().await;
        assert!(result.is_failed());
        Ok(())
    }
}
