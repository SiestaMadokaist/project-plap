#[cfg(feature = "datatransfer")]
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

use pkg::{
    civitai::{dto::model_version::ModelVersionDTO, typing::ModelCategory},
    id::InferenceModelId,
};

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait(?Send)]
pub trait InferenceModelProvider {
    // async fn model_detail(&self, id: &ModelId) -> anyhow::Result<ModelDetailDTO>;
    async fn get_detail(&self, id: &InferenceModelId) -> anyhow::Result<ModelVersionDTO>;

    fn abs_path(&self, id: &InferenceModelId, category: &ModelCategory, name: &str) -> PathBuf;

    #[cfg(feature = "datatransfer")]
    async fn download(&self, id: &InferenceModelId, dst: &Path) -> anyhow::Result<()>;
}

// Lets a `Box<dyn InferenceModelProvider>` be built from an `Rc<T>` without
// cloning `T` itself — needed when the same client instance is shared (e.g. a
// mock with expectations set once, boxed on every `civitai()` call).
#[async_trait::async_trait(?Send)]
impl<T: InferenceModelProvider> InferenceModelProvider for Rc<T> {
    async fn get_detail(&self, id: &InferenceModelId) -> anyhow::Result<ModelVersionDTO> {
        self.as_ref().get_detail(id).await
    }

    fn abs_path(&self, id: &InferenceModelId, category: &ModelCategory, name: &str) -> PathBuf {
        self.as_ref().abs_path(id, category, name)
    }

    #[cfg(feature = "datatransfer")]
    async fn download(&self, id: &InferenceModelId, dst: &Path) -> anyhow::Result<()> {
        self.as_ref().download(id, dst).await
    }
}
