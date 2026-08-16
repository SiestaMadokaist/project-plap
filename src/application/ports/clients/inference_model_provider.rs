use std::path::PathBuf;

use crate::pkg::{
    civitai::{dto::model_version::ModelVersionDTO, typing::ModelCategory},
    id::InferenceModelId,
};

#[async_trait::async_trait(?Send)]
pub trait InferenceModelProvider {
    // async fn model_detail(&self, id: &ModelId) -> anyhow::Result<ModelDetailDTO>;
    async fn get_detail(&self, id: &InferenceModelId) -> anyhow::Result<ModelVersionDTO>;

    fn abs_path(&self, id: &InferenceModelId, category: &ModelCategory, name: &str) -> PathBuf;

    #[cfg(feature = "datatransfer")]
    async fn download(&self, id: &InferenceModelId, dst: &PathBuf) -> anyhow::Result<()>;
}
