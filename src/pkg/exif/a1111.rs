use crate::pkg::exif::{
    exif::Exif,
    traits::{ExifError, ExifTraits, WebUI},
};

pub struct A1111 {}

#[derive(Default)]
pub struct A1111Memo {}

impl WebUI for A1111 {
    type Memo = A1111Memo;
}

impl ExifTraits for Exif<A1111> {
    fn positive(&self) -> Result<&str, ExifError> {
        todo!()
    }

    fn negative(&self) -> Result<&str, ExifError> {
        todo!()
    }

    fn checkpoint(&self) -> Result<&str, ExifError> {
        todo!();
    }
}
