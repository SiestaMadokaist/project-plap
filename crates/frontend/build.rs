use std::collections::HashMap;

/// Build-time config for the wasm bundle. Values come from `.env.frontend` at the
/// workspace root when it exists (local builds and `make frontend-deploy`), and
/// otherwise from the committed `.env.frontend.example` template — the only one
/// present in CI. None of them are secret; they end up in the shipped bundle.
/// An explicit process env var of the same name wins over the file, so
/// `make frontend-build` can still override the API base from a terraform output.
/// Missing keys fall back to the defaults in `src/env.rs`.
const KEYS: &[&str] = &["PLAP_API_BASE", "PLAP_MODEL_BUCKET", "PLAP_IO_BUCKET"];

fn main() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let local = format!("{root}/.env.frontend");
    let template = format!("{root}/.env.frontend.example");
    println!("cargo:rerun-if-changed={local}");
    println!("cargo:rerun-if-changed={template}");

    let from_file = std::fs::read_to_string(&local)
        .or_else(|_| std::fs::read_to_string(&template))
        .map(|s| parse_dotenv(&s))
        .unwrap_or_default();

    for key in KEYS {
        println!("cargo:rerun-if-env-changed={key}");
        let value = std::env::var(key)
            .ok()
            .or_else(|| from_file.get(*key).cloned());
        if let Some(value) = value {
            println!("cargo:rustc-env={key}={value}");
        }
    }
}

/// Minimal `KEY=value` parser — enough for our own `.env.frontend.example`. Blank
/// lines and `#` comments are skipped; surrounding single/double quotes are trimmed.
fn parse_dotenv(contents: &str) -> HashMap<String, String> {
    contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| line.split_once('='))
        .map(|(k, v)| {
            let v = v.trim().trim_matches(|c| c == '"' || c == '\'');
            (k.trim().to_string(), v.to_string())
        })
        .collect()
}
