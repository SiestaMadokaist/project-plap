use crate::application::ports::clients::{
    compute::ComputeEngines, diffusions::DiffusionClient, notification::NotificationClient,
    raws::RawsClient, storage::StorageClient, translator::TranslatorClient,
};

// pub trait ClientContainer {
//     type Translator: TranslatorClient;
//     type Raws: RawsClient;
//     type Storage: StorageClient;
//     type Notification: NotificationClient;
//     type Engines: ComputeEngines;

//     fn engines(&self) -> &Self::Engines;
//     fn translator(&self) -> &Self::Translator;
//     fn raws(&self) -> &Self::Raws;
//     fn storage(&self) -> &Self::Storage;
//     fn notification(&self) -> &Self::Notification;
//     fn diffusion(&self) -> &dyn DiffusionClient;
// }

pub trait HasTranslator {
    type Translator: TranslatorClient;
    fn translator(&self) -> &Self::Translator;
}
// impl_has!(HasTranslator, Translator, translator, ClientContainer);

pub trait HasRaws {
    type Raws: RawsClient;
    fn raws(&self) -> &Self::Raws;
}
// impl_has!(HasRaws, Raws, raws, ClientContainer);

pub trait HasStorage {
    type Storage: StorageClient;
    fn storage(&self) -> &Self::Storage;
}
// impl_has!(HasStorage, Storage, storage, ClientContainer);

pub trait HasNotification {
    type Notification: NotificationClient;
    fn notification(&self) -> &Self::Notification;
}
// impl_has!(HasNotification, Notification, notification, ClientContainer);
pub trait HasEngines {
    type Engines: ComputeEngines;
    fn engines(&self) -> &Self::Engines;
}

// impl_has!(HasEngines, Engines, engines, ClientContainer);

// Doesn't fit impl_has!: no associated type, returns a trait object instead
// (see the "runtime-chosen A1111 vs ComfyUI" discussion).
pub trait HasDiffusion {
    fn diffusion(&self) -> &dyn DiffusionClient;
}

// impl<CC: ClientContainer> HasDiffusion for CC {
//     fn diffusion(&self) -> &dyn DiffusionClient {
//         ClientContainer::diffusion(self)
//     }
// }

pub trait AllClients:
    HasNotification + HasRaws + HasStorage + HasTranslator + HasDiffusion + HasEngines
{
}
