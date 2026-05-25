use crate::application::ports::clients::{
    notification::NotificationClient, raws::RawsClient, storage::StorageClient,
    translator::TranslatorClient,
};

#[allow(async_fn_in_trait)]
pub trait ClientRepository {
    fn translator(&self) -> impl TranslatorClient;
    fn raws(&self) -> impl RawsClient;
    fn storage(&self) -> impl StorageClient;
    fn notification(&self) -> impl NotificationClient;
}
