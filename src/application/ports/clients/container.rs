use crate::application::ports::clients::translator::TranslatorClient;

pub trait ClientRepository {
    async fn translator(&self) -> impl TranslatorClient;
}
