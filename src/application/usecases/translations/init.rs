use std::rc::Rc;

use tokio::sync::OnceCell;

use crate::{
    application::{
        dto::translation::{TranslationDTO, TranslationResponse},
        ports::{
            clients::{raws::RawsClient, translator::TranslatorClient},
            repository::translation::TranslationRepository,
        },
        usecases::bases::Usecase,
    },
    domain::{
        errors::DomainError,
        translation::{ChapterId, NovelId, RawSource},
    },
};

pub struct Params {
    pub novel_id: NovelId,
    pub starting_chapter: Option<ChapterId>,
}

pub trait Repos {
    type TR: TranslationRepository;
    fn translation_repo(&self) -> &Self::TR;
}

pub trait Clients {
    type TC: TranslatorClient;
    type Raw: RawsClient;
    fn translator(&self) -> &Self::TC;
    fn raw(&self) -> &Self::Raw;
}

struct Memo {
    latest_raw: OnceCell<ChapterId>,
}

impl Memo {
    fn new() -> Self {
        return Memo {
            latest_raw: OnceCell::new(),
        };
    }
}

pub struct Init<R: Repos, C: Clients> {
    repo: Rc<R>,
    client: Rc<C>,
    params: Params,
    memo: Memo,
}

impl<R: Repos, C: Clients> Init<R, C> {
    pub fn new(repo: Rc<R>, client: Rc<C>, params: Params) -> Self {
        return Init {
            repo,
            client,
            params,
            memo: Memo::new(),
        };
    }

    async fn latest_raw(&self) -> Result<&ChapterId, DomainError> {
        let chapter = self
            .memo
            .latest_raw
            .get_or_try_init(async || {
                let ch = self.client.raw().latest(&self.params.novel_id).await;
                return ch;
            })
            .await?;
        return Ok(chapter);
    }

    async fn starting_chapter(&self) -> Result<&ChapterId, DomainError> {
        let starting_chapter = match self.params.starting_chapter.as_ref() {
            None => self.latest_raw().await,
            Some(x) => Ok(x),
        }?;
        return Ok(starting_chapter);
    }
}

impl<R: Repos, C: Clients> Usecase<TranslationResponse> for Init<R, C> {
    type Output = TranslationDTO;
    async fn exec(self) -> Result<TranslationDTO, DomainError> {
        let starting_chapter = self.starting_chapter().await?;
        let repo = self.repo.translation_repo();
        let init = repo
            .init(&self.params.novel_id, starting_chapter, RawSource::Syosetu)
            .await?;
        let dto = TranslationDTO::new(init);
        return Ok(dto);
    }
}
