use domain::ctx;
use reqwest::header::{HeaderMap, HeaderValue};

fn _internal_api(ctx: &ctx::Context, mut header: HeaderMap) -> reqwest::ClientBuilder {
    let b = reqwest::Client::builder();
    let trace_id = ctx.trace_id().clone();
    let trace_id_value = trace_id.to_string().parse::<HeaderValue>();
    if let Ok(value) = trace_id_value {
        header.insert("x-trace-id", value);
    }
    b.default_headers(header.clone())
}

pub fn internal_builder(ctx: &ctx::Context, additional: HeaderMap) -> reqwest::ClientBuilder {
    _internal_api(ctx, additional)
}
