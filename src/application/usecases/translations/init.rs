use std::rc::Rc;

use serde::Deserialize;
use tokio::sync::OnceCell;

use crate::{
    application::{
        dto::translation::{TranslationDTO, TranslationResponse},
        ports::{clients::raws::RawsClient, repository::translation::TranslationRepository},
        usecases::{
            bases::Usecase,
            translations::traits::{TLClients, TLRepos},
        },
    },
    domain::{
        errors::DomainError,
        translation::{ChapterId, NovelId, RawSource},
    },
};
#[derive(Deserialize)]
pub struct Params {
    pub novel_id: NovelId,
    pub starting_chapter: Option<ChapterId>,
    pub title: String,
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

pub struct Init<R: TLRepos, C: TLClients> {
    repo: Rc<R>,
    client: Rc<C>,
    params: Params,
    memo: Memo,
}

impl<R: TLRepos, C: TLClients> Init<R, C> {
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
                let raw_client = self.client.raws();
                let ch = raw_client.latest(&self.params.novel_id).await;
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

impl<R: TLRepos, C: TLClients> Usecase<TranslationResponse> for Init<R, C> {
    type Output = TranslationDTO;
    async fn exec(self) -> Result<TranslationDTO, DomainError> {
        let starting_chapter = self.starting_chapter().await?;
        let repo = self.repo.translation();
        let init = repo
            .init(
                &self.params.novel_id,
                starting_chapter,
                &self.params.title,
                RawSource::Syosetu,
            )
            .await?;
        let dto = TranslationDTO::new(init);
        return Ok(dto);
    }
}
