use std::cell::OnceCell;
pub mod nodes;
use crate::pkg::exif::{
    comfyui::nodes::ComfyWorkflow,
    exif::Exif,
    traits::{ExifError, ExifTraits, WebUI},
};

#[derive(Default)]
pub struct ComfyMemo {
    range: OnceCell<(usize, usize)>,
    workflow: OnceCell<Result<ComfyWorkflow, ExifError>>,
}

pub struct ComfyUI {}
impl WebUI for ComfyUI {
    type Memo = ComfyMemo;
}

impl Exif<ComfyUI> {
    pub fn new(data: Vec<u8>) -> Self {
        Exif::<ComfyUI> {
            memo: ComfyMemo {
                range: OnceCell::new(),
                workflow: OnceCell::new(),
            },
            data,
            _src: std::marker::PhantomData,
        }
    }

    pub fn workflow(&self) -> &Result<ComfyWorkflow, ExifError> {
        let memoized = self.memo.workflow.get_or_init(|| {
            let string = self.text();
            let wf: Result<ComfyWorkflow, ExifError> = serde_json::from_str(&string).map_err(|e| {
                tracing::error!("Parsing Failed: {}", e);
                ExifError::ParsingFailed
            });
            wf
        });
        memoized
    }

    fn get_str(&self) -> Result<&str, ExifError> {
        if !self.valid() {
            return Err(ExifError::InvalidRange);
        }
        let (from, to) = self.text_range();
        let text = std::str::from_utf8(&self.data[*from..*to]).map_err(|_| ExifError::NotExtracted);
        text
    }

    fn valid(&self) -> bool {
        let (_, to) = self.text_range();
        let zero: &usize = &0;
        to > zero
    }

    pub fn text(&self) -> String {
        let result = self.get_str();
        match result {
            Ok(s) => s.into(),
            Err(_) => "".into(),
        }
    }

    fn text_range(&self) -> &(usize, usize) {
        let memoized = self.memo.range.get_or_init(|| {
            let open: u8 = b'{';
            let close: u8 = b'}';
            let escape: u8 = b'/';
            let mut counter = 0;
            let mut start: usize = 0;
            let mut end: usize = 0;
            for (i, c) in self.data.iter().enumerate() {
                if (i > 1000) && (start == 0) {
                    break;
                }
                if c == &escape {
                    continue;
                }
                if c == &open {
                    if start == 0 {
                        start = i;
                    }
                    counter += 1
                }
                if c == &close {
                    counter -= 1;
                }
                if (counter == 0) && (start > 0) {
                    end = i + 1;
                    break;
                }
            }
            (start, end)
        });
        memoized
    }
}

impl ExifTraits for Exif<ComfyUI> {
    fn checkpoint(&self) -> Result<String, ExifError> {
        let wf = self.workflow().as_ref().map_err(|x| x.clone())?;
        let s: String = match wf.checkpoint() {
            None => "???".into(),
            Some(x) => x.clone(),
        };
        Ok(s)
    }
    fn negative(&self) -> Result<String, ExifError> {
        let wf = self.workflow().as_ref().map_err(|x| x.clone())?;
        let s: String = match wf.negative() {
            None => "-".into(),
            Some(x) => x.clone(),
        };
        Ok(s)
    }
    fn positive(&self) -> Result<String, ExifError> {
        let wf = self.workflow().as_ref().map_err(|x| x.clone())?;
        let s: String = match wf.positive() {
            None => "-".into(),
            Some(x) => x.clone(),
        };
        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_range_finds_the_json_span() -> std::io::Result<()> {
        let data = std::fs::read("./samples/inputs/saber.png")?;
        let exif = Exif::<ComfyUI>::new(data);
        let text = exif.text();
        print!("raw: {}\n", text);
        let checkpoint = exif.checkpoint()?;
        print!("checkpoint: {}\n", checkpoint);
        let positive = exif.positive()?;
        print!("positive: {}\n", positive);
        let negative = exif.negative()?;
        print!("negative: {}\n", negative);
        Ok(())
    }
}
