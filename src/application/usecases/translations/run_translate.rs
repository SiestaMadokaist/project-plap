use std::rc::Rc;

use async_openai::Chat;
use tokio::sync::OnceCell;

use crate::{
    application::{
        ports::{
            clients::{raws::RawsClient, translator::TranslatorClient},
            repository::translation::TranslationRepository,
        },
        usecases::errors::UsecaseError,
    },
    domain::translation::{ChapterId, NovelId, Translation},
};

pub struct GetLatest {
    novel_id: NovelId,
    chapter_id: ChapterId,
}

pub trait Repos {
    type TR: TranslationRepository;
    fn translation_repo(&self) -> &Self::TR;
}

pub trait Clients {
    type TC: TranslatorClient;
    type Raw: RawsClient;
    fn translator(&self) -> Self::TC;
    fn raw(&self) -> Self::Raw;
}

pub struct Params {
    id: NovelId,
}

pub struct RunTranslation<R: Repos, C: Clients> {
    repo: Rc<R>,
    client: Rc<C>,
    params: Params,
    latest_raw: OnceCell<ChapterId>,
}

impl<R: Repos, C: Clients> RunTranslation<R, C> {
    pub fn new(repo: Rc<R>, client: Rc<C>, params: Params) -> Self {
        return RunTranslation {
            repo,
            client,
            params,
            latest_raw: OnceCell::new(),
        };
    }

    async fn in_db(&self) -> Result<Option<Translation>, UsecaseError> {
        let translation_repo = self.repo.translation_repo();
        let result = translation_repo.latest(&self.params.id).await?;
        return Ok(result);
    }

    async fn is_translated(&self) -> Result<bool, UsecaseError> {
        let raw_id = self.client.raw().latest_chapter(&self.params.id).await?;
        let in_db = self.in_db().await?;
        let is_over: bool = match in_db {
            None => false,
            Some(x) => x.translated(raw_id),
        };
        return Ok(is_over);
    }
    // async fn exec(&self) -> Result<Translation, UsecaseError> {}
}
