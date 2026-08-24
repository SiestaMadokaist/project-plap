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
    fn checkpoint(&self) -> Result<&str, ExifError> {
        let wf = self.workflow().as_ref().map_err(|x| *x)?;
        let s = wf.checkpoint().unwrap_or("-");
        Ok(s)
    }
    fn negative(&self) -> Result<&str, ExifError> {
        let wf = self.workflow().as_ref().map_err(|x| *x)?;
        let s = wf.negative().unwrap_or("-");
        Ok(s)
    }
    fn positive(&self) -> Result<&str, ExifError> {
        let wf = self.workflow().as_ref().map_err(|x| *x)?;
        let s = wf.positive().unwrap_or("-");
        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;

    #[test]
    fn can_extract_exif() -> std::io::Result<()> {
        let data = std::fs::read("./samples/inputs/mocks/saber.comfy.png")?;
        let exif = Exif::<ComfyUI>::new(data);
        let text = exif.text();
        print!("raw: {}\n", text);
        let checkpoint = exif.checkpoint()?;
        print!("checkpoint: {}\n", checkpoint);
        assert_eq!(checkpoint, "cosplayillustriousmi_v20.safetensors");

        let positive = exif.positive()?;
        print!("positive: {}\n", positive);
        assert!(positive.starts_with("photorealistic"));
        assert!(positive.ends_with("fate series."));

        let negative = exif.negative()?;
        print!("negative: {}\n", negative);
        assert!(negative.starts_with("lowres, bad anatomy"));
        assert!(negative.ends_with("username, blurry"));

        Ok(())
    }

    #[test]
    fn invalid_doesnot_panic() -> std::io::Result<()> {
        let data = std::fs::read("./samples/inputs/mocks/elysia.a1111.png")?;
        let exif = Exif::<ComfyUI>::new(data);
        let text = exif.text();
        assert_eq!(text, "");
        let checkpoint = exif.checkpoint();
        assert_matches!(checkpoint, Err(ExifError::ParsingFailed));

        let positive = exif.positive();
        assert_matches!(positive, Err(ExifError::ParsingFailed));

        let negative = exif.negative();
        assert_matches!(negative, Err(ExifError::ParsingFailed));

        Ok(())
    }
}
