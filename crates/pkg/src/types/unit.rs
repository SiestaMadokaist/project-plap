use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Px(pub i32);

/** 0-based-index eg array */
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Index0(pub i32);

/** 1-based-index */
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Index1(pub i32);

impl Index1 {
    pub fn next(&mut self) {
        self.0 += 1;
    }
}

impl Index0 {
    pub fn next(&mut self) {
        self.0 += 1;
    }
}

pub const INDEX_ZERO: Index0 = Index0(0);
pub const INDEX_FIRST: Index1 = Index1(1);

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

#[cfg(test)]
mod tests {
    use crate::types::unit::{Index0, Index1, INDEX_FIRST, INDEX_ZERO};

    #[test]
    fn zero2one() -> std::io::Result<()> {
        let first: Index1 = INDEX_ZERO.into();
        assert_eq!(first.0, 1);
        assert_eq!(first, INDEX_FIRST);
        Ok(())
    }

    #[test]
    fn one2zero() -> std::io::Result<()> {
        let zero: Index0 = INDEX_FIRST.into();
        assert_eq!(zero.0, 0);
        assert_eq!(zero, INDEX_ZERO);
        Ok(())
    }
}
