use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Px(pub i32);

/** 0-based-index eg array */
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub struct Index0(pub i32);

/** 1-based-index */
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub struct Index1(pub i32);

impl Index1 {
    pub fn next(&mut self) -> () {
        self.0 += 1;
    }
}

impl Index0 {
    pub fn next(&mut self) -> () {
        self.0 += 1;
    }
}

impl From<Index1> for Index0 {
    fn from(value: Index1) -> Self {
        Index0(value.0 - 1)
    }
}

impl From<Index0> for Index1 {
    fn from(value: Index0) -> Self {
        Index1(value.0 + 1)
    }
}
