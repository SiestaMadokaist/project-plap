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
