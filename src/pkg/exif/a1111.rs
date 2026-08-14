use crate::pkg::exif::{
    exif::Exif,
    traits::{ExifTraits, WebUI},
};

pub struct A1111 {}

#[derive(Default)]
pub struct A1111Memo {}

impl WebUI for A1111 {
    type Memo = A1111Memo;
}

impl ExifTraits for Exif<A1111> {
    fn checkpoints(&self) -> &str {
        todo!()
    }
    fn negative(&self) -> &str {
        todo!()
    }
    fn positive(&self) -> &str {
        todo!()
    }
}
