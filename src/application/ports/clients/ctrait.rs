use crate::application::ports::clients::{
    diffusions::DiffusionClient, notification::NotificationClient, raws::RawsClient,
    storage::StorageClient, translator::TranslatorClient,
};

pub trait ClientContainer {
    type Translator: TranslatorClient;
    type Raws: RawsClient;
    type Storage: StorageClient;
    type Notification: NotificationClient;
    type Diffusion: DiffusionClient;
    fn translator(&self) -> &Self::Translator;
    fn raws(&self) -> &Self::Raws;
    fn storage(&self) -> &Self::Storage;
    fn notification(&self) -> &Self::Notification;
    fn diffusion(&self) -> &Self::Diffusion;
}

pub trait HasTranslator {
    type Translator: TranslatorClient;
    fn translator(&self) -> &Self::Translator;
}

impl<CC: ClientContainer> HasTranslator for CC {
    type Translator = CC::Translator;
    fn translator(&self) -> &Self::Translator {
        ClientContainer::translator(self)
    }
}

pub trait HasRaws {
    type Raws: RawsClient;
    fn raws(&self) -> &Self::Raws;
}

impl<CC: ClientContainer> HasRaws for CC {
    type Raws = CC::Raws;
    fn raws(&self) -> &Self::Raws {
        ClientContainer::raws(self)
    }
}

pub trait HasStorage {
    type Storage: StorageClient;
    fn storage(&self) -> &Self::Storage;
}

impl<CC: ClientContainer> HasStorage for CC {
    type Storage = CC::Storage;
    fn storage(&self) -> &Self::Storage {
        ClientContainer::storage(self)
    }
}

pub trait HasNotification {
    type Notification: NotificationClient;
    fn notification(&self) -> &Self::Notification;
}

impl<CC: ClientContainer> HasNotification for CC {
    type Notification = CC::Notification;
    fn notification(&self) -> &Self::Notification {
        ClientContainer::notification(self)
    }
}

pub trait HasDiffusion {
    type Diffusion: DiffusionClient;
    fn diffusion(&self) -> &Self::Diffusion;
}

impl<CC: ClientContainer> HasDiffusion for CC {
    type Diffusion = CC::Diffusion;
    fn diffusion(&self) -> &Self::Diffusion {
        ClientContainer::diffusion(self)
    }
}

pub trait AllClients:
    HasNotification + HasRaws + HasStorage + HasTranslator + HasDiffusion
{
}
