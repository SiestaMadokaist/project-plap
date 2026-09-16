use domain::{ctx, errors::DomainError};

pub fn internal_api(ctx: ctx::Context) -> Result<reqwest::Client, DomainError> {
    let b = reqwest::Client::builder();
    let header = reqwest::header::HeaderMap::new();
    let trace_id = ctx.trace_id().clone();
    header.insert("x-trace-id", trace_id.to_string().into());
    b.default_headers(header);
    b.build()
        .map_err(|x| DomainError::ValidationError(x.to_string()))
}
