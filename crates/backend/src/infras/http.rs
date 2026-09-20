use domain::{ctx, errors::DomainError};
use reqwest::header::HeaderValue;

fn _internal_api(ctx: &ctx::Context) -> Result<reqwest::Client, DomainError> {
    let b = reqwest::Client::builder();
    let header = &mut reqwest::header::HeaderMap::new();
    let trace_id = ctx.trace_id().clone();
    let trace_id_value = trace_id.to_string().parse::<HeaderValue>();
    if let Ok(value) = trace_id_value {
        header.insert("x-trace-id", value);
    }
    b.default_headers(header.clone())
        .build()
        .map_err(|x| DomainError::ValidationError(x.to_string()))
}

pub fn internal_api(ctx: &ctx::Context) -> reqwest::Client {
    _internal_api(ctx).unwrap_or_default()
}
