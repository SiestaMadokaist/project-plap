use crate::application::ports::clients::translator::TranslatorClient;

#[allow(async_fn_in_trait)]
pub trait ClientRepository {
    async fn translator(&self) -> impl TranslatorClient;
}
