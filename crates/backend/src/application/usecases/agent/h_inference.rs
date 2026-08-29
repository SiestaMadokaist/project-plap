use std::rc::Rc;

use crate::application::ports::clients::{
    container::{HasDiffusion, HasModelStorage},
    storage::StorageClient,
};
use domain::{
    commands::{command::Progression, inference::InferenceConfig},
    errors::DomainError,
};
use pkg::{exif::comfyui::nodes::ComfyWorkflow, macros::trait_clients};

trait_clients!(HandleInferenceClient, HasDiffusion, HasModelStorage);

pub struct HandleInference<'a, C: HandleInferenceClient> {
    clients: Rc<C>,
    progress: Progression,
    config: &'a InferenceConfig<String>,
}

impl<'a, C: HandleInferenceClient> HandleInference<'a, C> {
    pub fn new(clients: Rc<C>, progress: Progression, config: &'a InferenceConfig<String>) -> Self {
        Self {
            clients,
            progress,
            config,
        }
    }

    async fn generate(&mut self) -> Result<(), DomainError> {
        if self.progress.is_done() {
            return Ok(());
        }
        let expand = match &self.config.workflow_id {
            None => None,
            Some(path) => {
                let storage = self.clients.model_storage();
                let string = storage.read(path).await?;
                let deserialized: ComfyWorkflow = serde_json::from_str(&string)?;
                Some(deserialized)
            }
        };
        let diffusion = self.clients.diffusion();
        diffusion
            .generate(self.config, expand)
            .await
            .map_err(|x| DomainError::ApiError(x.to_string()))?;
        Ok(())
    }

    pub async fn exec(&mut self) -> Progression {
        let result = self.generate().await;
        match result {
            Ok(_p) => {
                self.progress.increment();
                self.progress.clone()
            }
            Err(_) => {
                self.progress.fail();
                self.progress.clone()
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::application::ports::clients::{
        diffusions::{DiffusionClient, MockDiffusionClient},
        storage::MockStorageClient,
    };
    use pkg::types::unit::{Index0, INDEX_ZERO};

    struct MockContainer {
        diffuser: Box<dyn DiffusionClient>,
        storage: MockStorageClient,
    }

    impl MockContainer {
        fn new(diffuser: MockDiffusionClient) -> Rc<Self> {
            let boxed: Box<dyn DiffusionClient> = Box::new(diffuser);
            let mut storage = MockStorageClient::new();
            storage.expect_read().returning(|_| {
                let text = r#"{"id": "xxx", "nodes": []}"#;
                Ok(text.into())
            });
            let s = Self {
                diffuser: boxed,
                storage,
            };
            Rc::new(s)
        }
    }

    impl HasModelStorage for MockContainer {
        type ModelStorage = MockStorageClient;
        fn model_storage(&self) -> &Self::ModelStorage {
            &self.storage
        }
    }

    impl HasDiffusion for MockContainer {
        fn diffusion(&self) -> &dyn DiffusionClient {
            self.diffuser.as_ref()
        }
    }

    fn cfg() -> InferenceConfig<String> {
        let buffer = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../samples/inputs/jsons/domain/commands/inference.json"
        ))
        .expect("cannot find inference.json");
        let config: InferenceConfig<String> = serde_json::from_slice(&buffer)
            .expect("cannot deserialize buffer to InferenceConfig<String>");
        config
    }

    #[tokio::test]
    async fn test_partial_ok() {
        let mut diffuser = MockDiffusionClient::new();
        diffuser.expect_generate().returning(|_, _| Ok(()));
        let container = MockContainer::new(diffuser);
        let progress = Progression::new(Index0(2), INDEX_ZERO);
        let config = cfg();
        let mut inference = HandleInference::new(container, progress, &config);
        let result = inference.exec().await;
        assert!(result.is_started());
        assert!(result.is_failed() == false);
        assert!(result.is_done() == false);
    }

    #[tokio::test]
    async fn test_partial_done1() {
        let mut diffuser = MockDiffusionClient::new();
        diffuser.expect_generate().returning(|_, _| Ok(()));
        let container = MockContainer::new(diffuser);
        let progress = Progression::new(Index0(1), INDEX_ZERO);
        let config = cfg();
        let mut inference = HandleInference::new(container, progress, &config);
        let result = inference.exec().await;
        assert!(result.is_started());
        assert!(result.is_failed() == false);
        assert!(result.is_done());
    }

    #[tokio::test]
    async fn test_partial_done2() {
        let mut diffuser = MockDiffusionClient::new();
        diffuser.expect_generate().returning(|_, _| Ok(()));
        let container = MockContainer::new(diffuser);
        let progress = Progression::new(Index0(2), Index0(1));
        let config = cfg();
        let mut inference = HandleInference::new(container, progress, &config);
        let result = inference.exec().await;
        assert!(result.is_started());
        assert!(result.is_failed() == false);
        assert!(result.is_done());
    }

    #[tokio::test]
    async fn test_failed() {
        let mut diffuser = MockDiffusionClient::new();
        diffuser
            .expect_generate()
            .returning(|_, _| Err(DomainError::RateLimited.into()));
        let container = MockContainer::new(diffuser);
        let progress = Progression::new(Index0(2), Index0(1));
        let config = cfg();
        let mut inference = HandleInference::new(container, progress, &config);
        let result = inference.exec().await;
        assert!(result.is_started());
        assert!(result.is_failed());
        assert!(result.is_done() == false);
    }
}
