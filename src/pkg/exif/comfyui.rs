use std::{cell::OnceCell, path::PathBuf};

use crate::pkg::exif::{
    exif::Exif,
    traits::{ExifTraits, WebUI},
};

#[derive(Default)]
pub struct ComfyMemo {
    range: OnceCell<(usize, usize)>,
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
            },
            data,
            _src: std::marker::PhantomData,
        }
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
                    end = i;
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
        let d = &exif.data;
        let range = exif.text_range();
        let s = range.0;
        let e = range.1;
        print!("picking range: from ({}) to ({})\n", s, e);
        print!(
            "it start with: '{}' and ends with: '{}'\n",
            d[s] as char, d[e] as char
        );
        let text = &d[s..e + 1];
        let str = std::str::from_utf8(text).expect("msg");
        // let (s: char , e: char) = (d[range.0].into(), d[range.1].into());
        print!("range: {}\n\n", str);
        Ok(())
    }
}
