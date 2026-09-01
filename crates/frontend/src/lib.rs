pub mod api;
pub mod app;
pub mod pages;
pub mod session;
pub mod wallet;

/// Base URL of the API this frontend talks to. Point it at your local
/// `cargo lambda watch` or the deployed API Gateway stage.
pub const API_BASE: &str = "http://127.0.0.1:9001";
