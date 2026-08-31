use dto::{resources::translations::InitPayload, response::Placeholder};
use tokio::sync::OnceCell;

use crate::application::{
    ports::{
        clients::raws::RawsClient, repository::translation::TranslationRepository,
        usecase::UsecaseAPI,
    },
    usecases::translations::traits::{TLClients, TLRepos},
};
use domain::{
    errors::DomainError,
    translation::{ChapterId, RawSource},
};

struct Memo {
    latest_raw: OnceCell<ChapterId>,
}

impl Memo {
    fn new() -> Self {
        Memo {
            latest_raw: OnceCell::new(),
        }
    }
}

pub struct Init<'a, R: TLRepos, C: TLClients> {
    repo: &'a R,
    client: &'a C,
    params: InitPayload,
    memo: Memo,
}

impl<'a, R: TLRepos, C: TLClients> Init<'a, R, C> {
    pub fn new(repo: &'a R, client: &'a C, params: InitPayload) -> Self {
        Init {
            repo,
            client,
            params,
            memo: Memo::new(),
        }
    }

    async fn latest_raw(&self) -> Result<&ChapterId, DomainError> {
        let chapter = self
            .memo
            .latest_raw
            .get_or_try_init(async || {
                let raw_client = self.client.raws();
                let ch = raw_client.latest(&self.params.novel_id).await;
                ch
            })
            .await?;
        Ok(chapter)
    }

    async fn starting_chapter(&self) -> Result<&ChapterId, DomainError> {
        let starting_chapter = match self.params.starting_chapter.as_ref() {
            None => self.latest_raw().await,
            Some(x) => Ok(x),
        }?;
        Ok(starting_chapter)
    }
}

impl<R: TLRepos, C: TLClients> UsecaseAPI<Placeholder> for Init<'_, R, C> {
    async fn exec(&self) -> Result<Placeholder, DomainError> {
        let starting_chapter = self.starting_chapter().await?;
        let repo = self.repo.translation();
        let _init = repo
            .init(
                &self.params.novel_id,
                starting_chapter,
                &self.params.title,
                RawSource::Syosetu,
            )
            .await?;
        Ok(Placeholder(200))
    }
}
