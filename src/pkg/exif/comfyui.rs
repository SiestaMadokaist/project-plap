use std::{cell::OnceCell, path::PathBuf};
pub mod nodes;
use crate::pkg::exif::{
    comfyui::nodes::ComfyWorkflow,
    exif::Exif,
    traits::{ExifTraits, WebUI},
};

#[derive(Default)]
pub struct ComfyMemo {
    range: OnceCell<(usize, usize)>,
    workflow: OnceCell<Result<ComfyWorkflow, serde_json::Error>>,
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

    pub fn workflow(&self) -> &Result<ComfyWorkflow, serde_json::Error> {
        let memoized = self.memo.workflow.get_or_init(|| {
            let string = self.text();
            let wf: Result<ComfyWorkflow, serde_json::Error> = serde_json::from_str(&string);
            wf
        });
        memoized
    }

    pub fn get_str(&self) -> &str {
        let (from, to) = self.text_range();
        let text = std::str::from_utf8(&self.data[*from..*to]).expect("trust me");
        text
    }

    pub fn text(&self) -> String {
        let str = self.get_str();
        let result = str.replace("\n", "");
        return result;
    }

    pub fn text_range(&self) -> &(usize, usize) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_range_finds_the_json_span() -> std::io::Result<()> {
        let data = std::fs::read("./samples/inputs/saber.png")?;
        let exif = Exif::<ComfyUI>::new(data);
        let text = exif.text();
        print!("raw: {}\n", text);
        match exif.workflow() {
            Err(e) => {
                print!("error: {}\n", e.to_string());
            }
            Ok(wf) => {
                let firstnode = wf.nodes.first().expect("trust me");
                print!("json: {}\n\n", firstnode.tipe);
            }
        }
        Ok(())
    }
}
