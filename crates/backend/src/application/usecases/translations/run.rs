use dto::response::Placeholder;
use tokio::sync::OnceCell;

use crate::application::{
    ports::{
        clients::{
            notification::NotificationClient, raws::RawsClient, storage::StorageClient,
            translator::TranslatorClient,
        },
        repository::translation::TranslationRepository,
        usecase::UsecaseAPI,
    },
    usecases::translations::traits::{TLClients, TLRepos},
};
use domain::{
    errors::DomainError,
    translation::{ChapterId, TranslationDomain},
};
struct Memo {
    latest_raw: OnceCell<ChapterId>,
    untranslated_chapters: OnceCell<Vec<ChapterId>>,
}

impl Memo {
    fn new() -> Self {
        Memo {
            latest_raw: OnceCell::new(),
            untranslated_chapters: OnceCell::new(),
        }
    }
}

pub struct Run<'a, R: TLRepos, C: TLClients> {
    repo: &'a R,
    client: &'a C,
    params: dto::resources::translations::RunPayload,
    memo: Memo,
}

impl<'a, R: TLRepos, C: TLClients> Run<'a, R, C> {
    pub fn new(
        repo: &'a R,
        client: &'a C,
        params: dto::resources::translations::RunPayload,
    ) -> Self {
        Run {
            repo,
            client,
            params,
            memo: Memo::new(),
        }
    }

    async fn latest_translation(&self) -> Result<Option<TranslationDomain>, DomainError> {
        let result = self
            .repo
            .translation()
            .latest(&self.params.novel_id)
            .await?;
        Ok(result)
    }

    async fn latest_raw(&self) -> Result<&ChapterId, DomainError> {
        let chapter = self
            .memo
            .latest_raw
            .get_or_try_init(async || self.client.raws().latest(&self.params.novel_id).await)
            .await?;
        Ok(chapter)
    }

    async fn untranslated(&self) -> Result<&Vec<ChapterId>, DomainError> {
        let result = self
            .memo
            .untranslated_chapters
            .get_or_try_init(|| async {
                let latest_raw = self.latest_raw().await?;
                let latest_translation = self.latest_translation().await?;
                let latest_translated_chapter = latest_translation
                    .as_ref()
                    .map(|x| x.chapter_id())
                    .unwrap_or(&ChapterId(0));
                Ok::<Vec<ChapterId>, DomainError>(
                    latest_translated_chapter.until(latest_raw).collect(),
                )
            })
            .await?;
        Ok(result)
    }

    async fn run_translation(&self, prev: &TranslationDomain) -> Result<Placeholder, DomainError> {
        tracing::info!(item = ?prev, "latest translation");
        let untranslated = self.untranslated().await?;
        let tl_repo = self.repo.translation();
        let storage = self.client.output_storage();
        let notification = self.client.notification();
        for chapter_id in untranslated {
            let raw = self
                .client
                .raws()
                .read(&self.params.novel_id, chapter_id)
                .await?;
            let translated = self.client.translator().translate(&raw).await?;
            let inserted = tl_repo.insert(prev, chapter_id).await?;
            storage
                .write(
                    &inserted.filepath(),
                    &translated.bytes().collect::<Vec<u8>>(),
                )
                .await?;
            let public_url = storage.public_url(&inserted.filepath());
            let message = format!(
                "chapter {} of {:?} has just been translated here: {}",
                inserted.title(),
                inserted.chapter_id(),
                public_url,
            );
            notification.notify(&message).await?;
        }
        Ok(Placeholder(200))
    }
}

impl<R: TLRepos, C: TLClients> UsecaseAPI<Placeholder> for Run<'_, R, C> {
    async fn exec(&self) -> Result<Placeholder, DomainError> {
        let latest = self.latest_translation().await?;
        match latest {
            None => Err(DomainError::Prerequisite("story not initialized".into())),
            Some(latest) => self.run_translation(&latest).await,
        }
    }
}
