use std::marker::PhantomData;

use crate::types::{error::FormatError, strings::StringFormat};
use serde::{Deserialize, Serialize};
pub trait HexPrefix {}
#[derive(Debug, Clone)]
/** zero prefixed hex, eg: 0xabcdef1234 */
pub struct Zx {}
impl HexPrefix for Zx {}

#[derive(Debug, Clone)]
pub struct Plain {}
impl HexPrefix for Plain {}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hex<P: HexPrefix> {
    pub s: String,
    p: PhantomData<P>,
}

impl Hex<Zx> {
    pub fn plain(&self) -> Hex<Plain> {
        let s = self.s.replacen("0x", "", 1);
        s.as_str()
            .try_into()
            .expect("Hex ZX should already a valid hex string")
    }
}

impl Hex<Plain> {
    pub fn zx(&self) -> Hex<Zx> {
        let s = format!("0x{}", self.s);
        s.as_str()
            .try_into()
            .expect("Hex plain should already be a valid plain hex string")
    }
}

/** this hex must start with 0x */
impl StringFormat for Hex<Zx> {
    fn validate(s: &str) -> Result<(), FormatError> {
        let valid_prefix = s.starts_with("0x");
        if !valid_prefix {
            let message = format!("must start with 0x, got {}", s);
            return Err(FormatError::InvalidPrefix(message));
        }
        // 0x is guaranteed to be first because of the earlier guard.
        let plain = s.replacen("0x", "", 1);
        let valid_content = plain.chars().all(|x| x.is_ascii_hexdigit());
        if !valid_content {
            let message = format!("content must only be 0x followed by hexadecimal, got {}", s);
            return Err(FormatError::InvalidContent(message));
        }
        Ok(())
    }

    fn as_str(&self) -> &str {
        &self.s
    }
    // pub fn to_bytes(&self) -> Result<Vec<u8>, hex::FromHexError> {
    //     hex::decode(&self.s)
    // }
}

/** this hex must start without 0x */
impl StringFormat for Hex<Plain> {
    fn validate(s: &str) -> Result<(), FormatError> {
        let valid = s.chars().all(|x| x.is_ascii_hexdigit());
        match valid {
            true => Ok(()),
            _ => {
                let message = format!("content must only be 0x followed by hexadecimal, got {}", s);
                Err(FormatError::InvalidContent(message))
            }
        }
    }

    fn as_str(&self) -> &str {
        &self.s
    }
}

impl TryFrom<&str> for Hex<Zx> {
    type Error = FormatError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Hex::<Zx>::validate(value)?;
        Ok(Self {
            s: value.to_lowercase(),
            p: std::marker::PhantomData,
        })
    }
}
impl TryFrom<&str> for Hex<Plain> {
    type Error = FormatError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Hex::<Plain>::validate(value)?;
        Ok(Self {
            s: value.to_lowercase(),
            p: std::marker::PhantomData,
        })
    }
}
