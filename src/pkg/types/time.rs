use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/*
 * ms since epoch
 */
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimestampMS(pub i64);

impl TimestampMS {
    pub fn now() -> Self {
        Self(Utc::now().timestamp_millis())
    }
}

impl TryFrom<TimestampMS> for DateTime<Utc> {
    type Error = ();
    fn try_from(value: TimestampMS) -> Result<Self, Self::Error> {
        DateTime::from_timestamp_millis(value.0).ok_or(())
    }
}

impl From<DateTime<Utc>> for TimestampMS {
    fn from(ts: DateTime<Utc>) -> Self {
        Self(ts.timestamp_millis())
    }
}

/*
 * seconds since epoch
 */
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Copy)]
pub struct Timestamp(pub i64);

impl Timestamp {
    pub fn now() -> Self {
        Self(Utc::now().timestamp())
    }

    pub fn sub(&self, other: &Timestamp) -> Second {
        let dt = self.0 - other.0;
        return Second(dt);
    }

    pub fn to_datestring(&self) -> String {
        let dt: DateTime<Utc> = (*self)
            .try_into()
            .expect("timestamp out of range for DateTime<Utc>");
        dt.format("%Y-%m-%d").to_string()
    }
}

impl TryFrom<Timestamp> for DateTime<Utc> {
    type Error = ();
    fn try_from(value: Timestamp) -> Result<Self, Self::Error> {
        DateTime::from_timestamp(value.0, 0).ok_or(())
    }
}

impl From<DateTime<Utc>> for Timestamp {
    fn from(ts: DateTime<Utc>) -> Self {
        Self(ts.timestamp())
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Second(pub i64);

impl Second {
    pub fn gt(&self, other: &Second) -> bool {
        self.0 > other.0
    }

    pub fn lt(&self, other: &Second) -> bool {
        self.0 < other.0
    }

    pub fn eq(&self, other: &Second) -> bool {
        self.0 == other.0
    }
}

impl From<i64> for Second {
    fn from(value: i64) -> Self {
        Second(value)
    }
}

impl From<&Second> for core::time::Duration {
    fn from(value: &Second) -> Self {
        std::time::Duration::from_secs(value.0 as u64)
    }
}
