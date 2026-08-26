use std::rc::Rc;

use domain::storage::{StoragePath, StoragePrefix};
use frontend::api::plap::PlapApi;
use leptos::prelude::*;
use pkg::types::strings::{JWT, URL};

// TODO: point at your deployed/local API (e.g. via `cargo lambda watch`)
const API_BASE: &str = "http://127.0.0.1:9001";

#[component]
fn ModelList(paths: Vec<StoragePath>) -> impl IntoView {
    view! {
        <div class="model-list">
            {paths
                .into_iter()
                .map(|path| view! { <div class="model">{path.0}</div> })
                .collect_view()}
        </div>
    }
}

#[component]
fn App() -> impl IntoView {
    let plap = PlapApi::new(JWT("test".into()), URL(API_BASE.into()));
    let rc = Rc::new(plap);
    let (prefix, set_prefix) = signal(StoragePrefix("comfyui".into()));
    let models = LocalResource::new(move || {
        let client = rc.clone();
        let value = prefix.get();
        async move { client.list_models(value).await }
    });
    view! {
        <main>
            <h1>"Project-Plap"</h1>
            <input
            type="text"
            prop:value={move || prefix.get().0 }
            on:input={move |ev| {
                let v = event_target_value(&ev);
                let p = StoragePrefix(v);
                set_prefix.set(p)
            }} />
            <Suspense fallback=|| view! { <p>"loading models..."</p> }>
                {move || Suspend::new(async move {
                    match models.await {
                        Ok(resp) => view! { <ModelList paths=resp.paths /> }.into_any(),
                        Err(err) => {
                            view! { <p class="error">"failed to load: " {err.to_string()}</p> }.into_any()
                        }
                    }
                })}
            </Suspense>
        </main>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
