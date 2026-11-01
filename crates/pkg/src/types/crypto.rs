use serde::{Deserialize, Serialize};

use crate::types::{
    error::FormatError,
    hex::{Hex, Plain, Zx},
    strings::StringFormat,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address<S: StringFormat>(S);

pub type Eth0x = Address<Hex<Zx>>;
pub type EthPlain = Address<Hex<Plain>>;
pub type HexSign = Hex<Zx>;

impl<S: StringFormat> Address<S> {
    pub fn get(&self) -> &S {
        &self.0
    }
    pub fn eth(s: &str) -> Result<Eth0x, FormatError> {
        let hex: Hex<Zx> = s.try_into()?;
        let a = Address(hex);
        Ok(a)
    }
}
