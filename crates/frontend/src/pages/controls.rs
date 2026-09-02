use domain::{
    errors::DomainError,
    storage::{StoragePath, StoragePrefix},
};
use leptos::prelude::*;
use pkg::{auth::claims::JWT, types::strings::URL};
use wasm_bindgen_futures::spawn_local;

use crate::{api::plap::PlapApi, session, API_BASE};

/// The two prefixes the panels list. Trailing slash keeps the match tight
/// (so `loras/` doesn't also pull in a sibling like `loras_xl/`).
const DIFFUSION_MODELS: &str = "comfyui/diffusion_models/";
const LORAS: &str = "comfyui/loras/";

/// A flat, recursive listing of everything under `prefix`. Clicking a row queues a
/// copy of that object to `models/<rest>` via `/agents/command/cp`. Reused for both
/// columns.
#[component]
fn ModelList(
    api: PlapApi,
    prefix: &'static str,
    set_session: WriteSignal<Option<JWT>>,
) -> impl IntoView {
    let (status, set_status) = signal(String::new());

    let items = LocalResource::new({
        let api = api.clone();
        move || {
            let api = api.clone();
            async move { api.list_models(StoragePrefix(prefix.into()), true).await }
        }
    });

    view! {
        <section class="panel">
            <h2 class="panel-title">{prefix}</h2>

            {move || {
                let s = status.get();
                (!s.is_empty()).then(|| view! { <p class="panel-status">{s}</p> })
            }}

            <Suspense fallback=|| {
                view! { <p class="muted panel-note">"Loading…"</p> }
            }>
                {
                    let api = api.clone();
                    move || {
                        let api = api.clone();
                        Suspend::new(async move {
                            match items.await {
                                Ok(resp) => {
                                    let bucket = resp.bucket;
                                    let paths = resp.tree.paths;
                                    if paths.is_empty() {
                                        return view! {
                                            <p class="muted panel-note">"Nothing here."</p>
                                        }
                                            .into_any();
                                    }
                                    let rows = paths
                                        .into_iter()
                                        .map(|p| {
                                            let api = api.clone();
                                            let bucket = bucket.clone();
                                            let key = p.0;
                                            let name = key
                                                .strip_prefix(prefix)
                                                .unwrap_or(&key)
                                                .to_string();
                                            let on_click = move |_| {
                                                let api = api.clone();
                                                let bucket = bucket.clone();
                                                let key = key.clone();
                                                set_status.set(format!("Queuing {key}…"));
                                                spawn_local(async move {
                                                    let msg = match api
                                                        .cp_model(bucket, StoragePath(key.clone()))
                                                        .await
                                                    {
                                                        Ok(_) => format!("Queued {key}"),
                                                        Err(e) => format!("Failed {key}: {e}"),
                                                    };
                                                    set_status.set(msg);
                                                });
                                            };
                                            view! {
                                                <li
                                                    class="item"
                                                    title="Promote to models/"
                                                    on:click=on_click
                                                >
                                                    {name}
                                                </li>
                                            }
                                        })
                                        .collect_view();
                                    view! { <ul class="item-list">{rows}</ul> }.into_any()
                                }
                                Err(DomainError::NotAllowed(_)) => {
                                    request_animation_frame(move || {
                                        session::clear();
                                        set_session.set(None);
                                    });
                                    view! {
                                        <p class="muted panel-note">
                                            "Session expired — returning to sign in…"
                                        </p>
                                    }
                                        .into_any()
                                }
                                Err(err) => {
                                    view! {
                                        <p class="auth-error">{format!("Failed to load: {err}")}</p>
                                    }
                                        .into_any()
                                }
                            }
                        })
                    }
                }
            </Suspense>
        </section>
    }
}

#[component]
pub fn Controls(jwt: JWT, set_session: WriteSignal<Option<JWT>>) -> impl IntoView {
    let api = PlapApi::new(jwt, URL(API_BASE.to_string()));

    let logout = move |_| {
        session::clear();
        set_session.set(None);
    };

    view! {
        <main class="dash">
            <header class="dash-bar">
                <span class="brand">
                    <span class="brand-mark">"⬡"</span>
                    <span class="brand-name">"Project-Plap"</span>
                </span>
                <button class="ghost-btn" on:click=logout>
                    "Log out"
                </button>
            </header>

            <section class="dash-body wide">
                <h1>"Controls"</h1>
                <div class="panels">
                    <ModelList api=api.clone() prefix=DIFFUSION_MODELS set_session />
                    <ModelList api=api.clone() prefix=LORAS set_session />
                </div>
            </section>
        </main>
    }
}
