pub trait WebUI {
    type Memo: Default;
}

pub(super) trait ExifTraits {
    fn positive(&self) -> &str;
    fn negative(&self) -> &str;
    fn checkpoints(&self) -> &str;
}
