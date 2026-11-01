use serde::{Deserialize, Serialize};

use crate::{
    displayable,
    types::{
        error::FormatError,
        hex::{zx, Hex, Plain, Zx},
        strings::StringFormat,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address<S: StringFormat>(S);

pub type Eth0x = Address<Hex<Zx>>;
pub type EthPlain = Address<Hex<Plain>>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EthSign(Hex<Zx>);
displayable!(EthSign);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EthPriv(Hex<Zx>);
displayable!(EthPriv);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EthPub(Hex<Zx>);
displayable!(EthPub);

impl TryFrom<&str> for EthSign {
    type Error = FormatError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let z = zx(value)?;
        Ok(Self(z))
    }
}

impl TryFrom<&str> for EthPriv {
    type Error = FormatError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let z = zx(value)?;
        Ok(Self(z))
    }
}

impl TryFrom<&str> for EthPub {
    type Error = FormatError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let z = zx(value)?;
        Ok(Self(z))
    }
}

impl EthSign {
    pub fn as_hex(&self) -> &Hex<Zx> {
        &self.0
    }
}

impl EthPriv {
    pub fn as_hex(&self) -> &Hex<Zx> {
        &self.0
    }
}

impl EthPub {
    pub fn as_hex(&self) -> &Hex<Zx> {
        &self.0
    }
}

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
