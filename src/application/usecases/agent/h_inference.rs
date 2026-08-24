use std::rc::Rc;

use crate::{
    application::ports::clients::container::HasDiffusion,
    domain::{
        commands::{command::Progression, inference::InferenceConfig},
        errors::DomainError,
    },
    pkg::macros::trait_clients,
};

trait_clients!(HandleInferenceClient, HasDiffusion);

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
        let diffusion = self.clients.diffusion();
        diffusion
            .generate(self.config)
            .await
            .map_err(|x| DomainError::ApiError(x.to_string()))?;
        Ok(())
    }

    pub async fn exec(&mut self) -> Progression {
        let result = self.generate().await;
        match result {
            Ok(p) => {
                self.progress.increment();
                self.progress
            }
            Err(_) => {
                self.progress.fail();
                self.progress
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        application::ports::clients::diffusions::{DiffusionClient, MockDiffusionClient},
        pkg::types::unit::{Index0, INDEX_ZERO},
    };

    struct MockContainer {
        diffuser: Box<dyn DiffusionClient>,
    }

    impl MockContainer {
        fn new(diffuser: MockDiffusionClient) -> Rc<Self> {
            let boxed: Box<dyn DiffusionClient> = Box::new(diffuser);
            let s = Self { diffuser: boxed };
            Rc::new(s)
        }
    }

    impl HasDiffusion for MockContainer {
        fn diffusion(&self) -> &dyn DiffusionClient {
            self.diffuser.as_ref()
        }
    }

    fn cfg() -> InferenceConfig<String> {
        let buffer = std::fs::read("./samples/inputs/jsons/domain/commands/inference.json")
            .expect("cannot find network2.json");
        let config: InferenceConfig<String> = serde_json::from_slice(&buffer)
            .expect("cannot deserialize buffer to InferenceConfig<String>");
        config
    }

    #[tokio::test]
    async fn test_partial_ok() {
        let mut diffuser = MockDiffusionClient::new();
        diffuser.expect_generate().returning(|_| Ok(()));
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
        diffuser.expect_generate().returning(|_| Ok(()));
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
        diffuser.expect_generate().returning(|_| Ok(()));
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
            .returning(|_| Err(DomainError::RateLimited.into()));
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
