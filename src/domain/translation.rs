#[derive(PartialEq, PartialOrd)]
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

pub type OptionalChapterId = Option<ChapterId>;

pub struct NovelId(pub String);

#[derive(PartialEq, Eq)]
pub enum RawSource {
    Syosetu,
}
#[derive(PartialEq, Eq, PartialOrd, Ord)]
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

pub struct TranslationProgress {
    latest_chapter: ChapterId,
    novel_id: NovelId,
    title: String,
    source: RawSource,
    status: Status,
}

impl TranslationProgress {
    pub fn new(
        novel_id: NovelId,
        chapter: ChapterId,
        title: &str,
        source: Option<RawSource>,
    ) -> Self {
        return TranslationProgress {
            latest_chapter: chapter,
            novel_id: novel_id,
            title: String::from(title),
            source: source.unwrap_or(RawSource::Syosetu),
            status: Status::Raw,
        };
    }

    pub fn latest_chapter(&self) -> &ChapterId {
        return &self.latest_chapter;
    }

    pub fn is_untranslated(&self, c: &ChapterId) -> bool {
        if self.latest_chapter < *c {
            return true;
        }
        if self.latest_chapter == *c {
            return self.status < Status::Processing;
        }
        return false;
    }
}
