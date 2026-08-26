use crate::application::ports::clients::{
    compute::ComputeEngines, compute_agent::ComputeAgent, diffusions::DiffusionClient,
    inference_model_provider::InferenceModelProvider, notification::NotificationClient,
    raws::RawsClient, storage::StorageClient, translator::TranslatorClient,
};

pub trait HasTranslator {
    type Translator: TranslatorClient;
    fn translator(&self) -> &Self::Translator;
}

pub trait HasRaws {
    type Raws: RawsClient;
    fn raws(&self) -> &Self::Raws;
}

pub trait HasModelStorage {
    type ModelStorage: StorageClient;
    fn model_storage(&self) -> &Self::ModelStorage;
}

pub trait HasOutputStorage {
    type OutputStorage: StorageClient;
    fn output_storage(&self) -> &Self::OutputStorage;
}

pub trait HasNotification {
    type Notification: NotificationClient;
    fn notification(&self) -> &Self::Notification;
}
pub trait HasEngines {
    type Engines: ComputeEngines;
    fn engines(&self) -> &Self::Engines;
}

// Doesn't fit impl_has!: no associated type, returns a trait object instead
// (see the "runtime-chosen A1111 vs ComfyUI" discussion).
pub trait HasDiffusion {
    fn diffusion(&self) -> &dyn DiffusionClient;
}

pub trait HasComputeAgent {
    fn agent(&self) -> &dyn ComputeAgent;
}

pub trait HasInferenceModelProvider {
    fn inference_model_provider(&self) -> &dyn InferenceModelProvider;
}
