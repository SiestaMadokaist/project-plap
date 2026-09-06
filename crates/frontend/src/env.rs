//! Build-time configuration, baked into the wasm bundle.
//!
//! Values are read from `.env.frontend.example` at the workspace root by `build.rs`
//! and passed in via `option_env!`. Nothing here is secret — it all ships to the
//! browser. Each field falls back to the dev default below when its key is unset.

/// The frontend's compiled-in config. Access through [`ENV`].
pub struct Env {
    /// Base URL of the API this frontend talks to. Default targets a local
    /// `cargo lambda watch`.
    pub api_base: &'static str,
    /// S3 bucket holding the `comfyui/…` model tree the control panels list and copy from.
    pub model_bucket: &'static str,
    /// S3 bucket for generated input/output artifacts.
    pub io_bucket: &'static str,
}

pub const ENV: Env = Env {
    api_base: match option_env!("PLAP_API_BASE") {
        Some(v) => v,
        None => "http://127.0.0.1:9001",
    },
    model_bucket: match option_env!("PLAP_MODEL_BUCKET") {
        Some(v) => v,
        None => "virginia-ramadoka",
    },
    io_bucket: match option_env!("PLAP_IO_BUCKET") {
        Some(v) => v,
        None => "ap3.ramadoka.com",
    },
};
