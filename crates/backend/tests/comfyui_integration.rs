//! Template for infra-adapter integration tests against a *live* external
//! dependency: drive the adapter (`ComfyUI`) through its public trait
//! (`DiffusionClient`), pointed at a real server via env var, using only the
//! crate's public API — same boundary `backend`'s own callers see.
//! Copy this file's shape (env-var URL + fixture-from-JSON + #[ignore]) for
//! the next `infras::*` adapter that needs a live counterpart to test against.

use backend::application::ports::clients::diffusions::DiffusionClient;
use backend::infras::diffusions::comfyui::ComfyUI;
use domain::commands::inference::InferenceConfig;
use pkg::comfy;
use pkg::types::strings::URL;
use pkg::utils::testhelper;

/// Requires a real ComfyUI server reachable at `COMFYUI_URL`
/// (e.g. `http://127.0.0.1:8188`). Not run by `cargo test` on its own —
/// start the server, then run `cargo test -- --ignored`.
#[ignore = "requires a live ComfyUI server at COMFYUI_URL"]
#[tokio::test]
async fn generate_against_live_server() {
    let v = 1;
    tracing::debug!("{}", v);
    let comfy_url = URL("http://127.0.0.1:8188".into());
    let client = ComfyUI::new(comfy_url);
    let config = testhelper::read_json::<InferenceConfig<String>>(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../samples/inputs/jsons/domain/commands/inference.json"
    ))
    .expect("should not error");
    let workflow = testhelper::read_json::<comfy::workflow::PostRequests>(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../samples/inputs/jsons/infras/comfyui/post-request.json"
    ))
    .expect("should not error");

    let result = client.generate(&config, Some(workflow)).await;

    // `generate` is still a stub: it always errors after reading the response
    // body. Once it returns something real, assert on that instead.
    dbg!(&result);
    assert!(result.is_err());
}
