use std::ops::Sub;

use chrono::{DateTime, Utc};
use pkg::{auth::claims::AuthClaims, id::TraceId, types::time::MilliSecond};

#[derive(Debug, Clone)]
pub enum CtxAuth {
    Authorized(AuthClaims),
    Guest,
}

#[derive(Debug, Clone)]
pub struct Context {
    auth: CtxAuth,
    trace_id: TraceId,
    start_at: DateTime<Utc>,
}

impl Context {
    pub fn guest(mtrace: Option<TraceId>) -> Self {
        let trace_id = mtrace.unwrap_or_default();
        let start_at = chrono::Utc::now();
        Self {
            auth: CtxAuth::Guest,
            trace_id,
            start_at,
        }
    }
    pub fn new(auth: AuthClaims, mtrace: Option<TraceId>) -> Self {
        let trace_id = mtrace.unwrap_or_default();
        let start_at = chrono::Utc::now();
        Self {
            auth: CtxAuth::Authorized(auth),
            trace_id,
            start_at,
        }
    }

    pub fn auth(&self) -> Option<&AuthClaims> {
        match &self.auth {
            CtxAuth::Authorized(x) => Some(x),
            CtxAuth::Guest => None,
        }
    }

    pub fn trace_id(&self) -> &TraceId {
        &self.trace_id
    }

    pub fn duration(&self, finished_at: DateTime<Utc>) -> MilliSecond {
        let diff = finished_at.sub(&self.start_at);
        diff.into()
    }
}
