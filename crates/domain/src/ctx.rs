use pkg::{auth::claims::AuthClaims, id::TraceId};

pub struct Context {
    auth: AuthClaims,
    trace_id: TraceId,
}

impl Context {
    fn new_trace_id() -> TraceId {
        TraceId("todo".into())
    }

    pub fn new(auth: AuthClaims, mtrace_id: Option<TraceId>) -> Self {
        let trace_id = mtrace_id.unwrap_or_else(Context::new_trace_id);
        Self { auth, trace_id }
    }

    pub fn auth(&self) -> &AuthClaims {
        &self.auth
    }

    pub fn trace_id(&self) -> &TraceId {
        &self.trace_id
    }
}
