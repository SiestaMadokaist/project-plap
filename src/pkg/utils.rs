use std::env;

use crate::pkg::types::time::Second;

pub fn var_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

pub fn var_second(key: &str) -> Second {
    let err = format!("{} must be set to a valid number", key);
    let s = env::var(key).expect(&err);
    let opti: Option<i64> = s.parse().ok();
    let i = opti.expect(&err);
    Second(i)
}

#[cfg(test)]
pub mod testhelper {
    #[derive(Debug)]
    pub enum Error {
        OpenFile(String),
        Serialize(String),
    }
    pub fn read_json<X: serde::de::DeserializeOwned>(path: &str) -> Result<X, Error> {
        let buffer = std::fs::read(path).map_err(|x| Error::OpenFile(x.to_string()))?;
        let command: X =
            serde_json::from_slice(&buffer).map_err(|x| Error::Serialize(x.to_string()))?;
        Ok(command)
    }
}
