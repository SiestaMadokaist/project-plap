#[derive(PartialEq, PartialOrd)]
pub struct ChapterId(pub i32);
pub struct NovelId(pub String);

#[derive(PartialEq, Eq)]
pub enum RawSource {
    Syosetu,
}

pub enum Status {
    Pending,
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

pub struct Translation {
    latest_chapter: ChapterId,
    novel_id: NovelId,
    name: String,
    source: RawSource,
    status: Status,
}

impl Translation {
    pub fn translated(&self, c: ChapterId) -> bool {
        return self.latest_chapter > c;
    }
}
