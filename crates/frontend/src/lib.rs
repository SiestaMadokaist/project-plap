pub mod api;
pub mod app;
pub mod pages;
pub mod session;
pub mod wallet;

/// Base URL of the API this frontend talks to.
///
/// Set `PLAP_API_BASE` at build time to point at a deployed API
/// (`make frontend-build` wires it from `terraform output api_url`); with no
/// override it targets a local `cargo lambda watch`.
pub const API_BASE: &str = match option_env!("PLAP_API_BASE") {
    Some(v) => v,
    None => "http://127.0.0.1:9001",
};
