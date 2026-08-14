use crate::pkg::exif::traits::WebUI;

pub struct Exif<S: WebUI> {
    pub(super) data: Vec<u8>,
    pub(super) memo: S::Memo,
    pub(super) _src: std::marker::PhantomData<S>,
}
