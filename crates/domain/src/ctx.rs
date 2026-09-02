use std::ops::Sub;

use chrono::{DateTime, Utc};
use pkg::{auth::claims::AuthClaims, id::TraceId, types::time::MilliSecond};

pub struct Context {
    auth: AuthClaims,
    trace_id: TraceId,
    start_at: DateTime<Utc>,
}

impl Context {
    fn new_trace_id() -> TraceId {
        TraceId("todo".into())
    }

    pub fn new(auth: AuthClaims, mtrace_id: Option<TraceId>) -> Self {
        let trace_id = mtrace_id.unwrap_or_else(Context::new_trace_id);
        let start_at = chrono::Utc::now();
        Self {
            auth,
            trace_id,
            start_at,
        }
    }

    pub fn auth(&self) -> &AuthClaims {
        &self.auth
    }

    pub fn trace_id(&self) -> &TraceId {
        &self.trace_id
    }

    pub fn duration(&self, finished_at: DateTime<Utc>) -> MilliSecond {
        let diff = finished_at.sub(&self.start_at);
        diff.into()
    }
}
