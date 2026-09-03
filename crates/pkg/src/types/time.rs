use std::time::Duration;

use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};

/*
 * ms since epoch
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimestampMS(pub i64);

impl TimestampMS {
    pub fn now() -> Self {
        Self(Utc::now().timestamp_millis())
    }
    pub fn utc(&self) -> Option<chrono::DateTime<Utc>> {
        let p = self.0;
        let sec = p / 1000;
        let nsec: u32 = (p % 1_000_000) as u32;
        DateTime::from_timestamp(sec, nsec)
    }
    pub fn to_datestring(&self) -> Option<String> {
        let dt = self.utc()?;
        let formatted = dt.format("%Y-%m-%d");
        Some(formatted.to_string())
    }
}

impl From<DateTime<Utc>> for TimestampMS {
    fn from(value: DateTime<Utc>) -> Self {
        let ms = value.timestamp_millis();
        TimestampMS(ms)
    }
}

/*
 * use Timestamp or TimestampMS for data store type
 * eg: database field, json response, etc.
 *
 * use chrono::DateTime<Utc> for data manipulation type
 * add/compare/abs/diff etc
 *
 * Timestamp represent seconds since epoch
 * TimestampMS represent milliseconds since epoch
 */
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Copy)]
pub struct Timestamp(pub i64);

impl Timestamp {
    pub fn now() -> Self {
        Self(chrono::Utc::now().timestamp())
    }

    pub fn utc(&self) -> Option<chrono::DateTime<Utc>> {
        DateTime::from_timestamp(self.0, 0)
    }

    pub fn to_datestring(&self) -> Option<String> {
        let dt = self.utc()?;
        let formatted = dt.format("%Y-%m-%d");
        Some(formatted.to_string())
    }
}

impl From<DateTime<Utc>> for Timestamp {
    fn from(value: DateTime<Utc>) -> Self {
        Timestamp(value.timestamp())
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Second(pub i64);
impl Second {
    pub fn to_delta(&self) -> TimeDelta {
        TimeDelta::seconds(self.0)
    }

    pub fn to_duration(&self) -> Duration {
        Duration::from_secs(self.0 as u64)
    }
}

pub struct MilliSecond(pub i64);
impl MilliSecond {
    pub fn to_delta(&self) -> TimeDelta {
        TimeDelta::milliseconds(self.0)
    }
}

impl From<chrono::TimeDelta> for Second {
    fn from(value: chrono::TimeDelta) -> Self {
        let i = value.num_seconds();
        Self(i)
    }
}

impl From<chrono::TimeDelta> for MilliSecond {
    fn from(value: chrono::TimeDelta) -> Self {
        let i = value.num_milliseconds();
        Self(i)
    }
}
