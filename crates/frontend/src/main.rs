use domain::commands::compute::ComputeRegion;
use gloo_net::http::Request;
use leptos::prelude::*;

// TODO: point at your deployed/local API (e.g. via `cargo lambda watch`)
const API_BASE: &str = "http://127.0.0.1:9000";

async fn fetch_region(path: &str) -> Result<ComputeRegion, String> {
    async {
        Request::get(&format!("{API_BASE}{path}"))
            .send()
            .await?
            .json::<ComputeRegion>()
            .await
    }
    .await
    .map_err(|e: gloo_net::Error| e.to_string())
}

#[component]
fn App() -> impl IntoView {
    let region = LocalResource::new(|| fetch_region("/compute/region"));

    view! {
        <main>
            <h1>"rust.lambda"</h1>
            <p>
                {move || match region.get().as_deref() {
                    None => "loading...".to_string(),
                    Some(Ok(region)) => format!("region: {region}"),
                    Some(Err(err)) => format!("failed to load region: {err}"),
                }}
            </p>
        </main>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
