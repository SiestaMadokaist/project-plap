use serde::{Deserialize, Serialize};

use crate::{
    domain::storage::StoragePath,
    pkg::types::time::{self, Timestamp},
};

#[derive(PartialEq, PartialOrd, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ChapterId(pub i32);

impl ChapterId {
    pub fn next(&self) -> ChapterId {
        return ChapterId(self.0 + 1);
    }

    pub fn until(&self, target: &ChapterId) -> impl Iterator<Item = ChapterId> {
        let start = self.0 + 1;
        let end = target.0;
        let iter = (start..=end).map(ChapterId);
        return iter;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NovelId(pub String);

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
pub enum RawSource {
    Syosetu,
}
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize, Deserialize)]
pub enum Status {
    Raw,
    Processing,
    Translated,
}

impl std::fmt::Display for RawSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RawSource::Syosetu => write!(f, "syosetu"),
        }
    }
}

pub trait TranslationGetter {
    fn title(&self) -> &String;
    fn created_at(&self) -> &Timestamp;
    fn updated_at(&self) -> &Timestamp;
    fn chapter_id(&self) -> &ChapterId;
    fn novel_id(&self) -> &NovelId;
    fn source(&self) -> &RawSource;
    fn status(&self) -> &Status;
}

#[derive(Clone)]
pub struct TranslationDomain {
    chapter_id: ChapterId,
    novel_id: NovelId,
    title: String,
    source: RawSource,
    status: Status,
    created_at: Timestamp,
    updated_at: Timestamp,
}

impl TranslationDomain {
    pub fn new(
        novel_id: NovelId,
        chapter: ChapterId,
        title: &str,
        source: Option<RawSource>,
    ) -> Self {
        return TranslationDomain {
            chapter_id: chapter,
            novel_id: novel_id,
            title: String::from(title),
            source: source.unwrap_or(RawSource::Syosetu),
            status: Status::Raw,
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        };
    }

    pub fn title(&self) -> &String {
        return &self.title;
    }

    pub fn status(&self) -> &Status {
        return &self.status;
    }

    pub fn created_at(&self) -> &Timestamp {
        return &self.created_at;
    }

    pub fn updated_at(&self) -> &Timestamp {
        return &self.updated_at;
    }

    pub fn chapter_id(&self) -> &ChapterId {
        return &self.chapter_id;
    }

    pub fn novel_id(&self) -> &NovelId {
        return &self.novel_id;
    }

    pub fn source(&self) -> &RawSource {
        return &self.source;
    }

    pub fn filepath(&self) -> StoragePath {
        let str = format!("{}/{}.txt", self.novel_id.0, self.chapter_id.0);
        return StoragePath(str);
    }
}

impl<T: TranslationGetter> From<T> for TranslationDomain {
    fn from(tl: T) -> Self {
        Self {
            novel_id: tl.novel_id().clone(),
            chapter_id: *tl.chapter_id(),
            source: *tl.source(),
            title: tl.title().clone(),
            created_at: *tl.created_at(),
            updated_at: *tl.updated_at(),
            status: tl.status().clone(),
        }
    }
}
