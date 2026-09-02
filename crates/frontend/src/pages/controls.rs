use std::collections::HashMap;

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

/// One clickable entry: every object that shares a path once the extension is
/// stripped (e.g. `…/render_studio-v2.0.{safetensors,jpg,json}`).
struct Group {
    /// stem relative to the panel prefix, e.g. `3188571/render_studio-v2.0`
    name: String,
    /// formats present, in listing order
    exts: Vec<String>,
    /// full s3 keys — one cp command is queued per key on click
    keys: Vec<String>,
}

/// Collapse a flat key listing into per-stem groups.
fn collapse(prefix: &str, paths: Vec<StoragePath>) -> Vec<Group> {
    let mut order: Vec<String> = Vec::new();
    let mut groups: HashMap<String, Group> = HashMap::new();

    for p in paths {
        let key = p.0;
        let (stem, ext) = match key.rsplit_once('.') {
            Some((s, e)) => (s.to_string(), Some(e.to_string())),
            None => (key.clone(), None),
        };

        let group = groups.entry(stem.clone()).or_insert_with(|| {
            order.push(stem.clone());
            Group {
                name: stem.strip_prefix(prefix).unwrap_or(&stem).to_string(),
                exts: Vec::new(),
                keys: Vec::new(),
            }
        });
        if let Some(e) = ext {
            if !group.exts.contains(&e) {
                group.exts.push(e);
            }
        }
        group.keys.push(key);
    }

    order.sort();
    order.into_iter().filter_map(|s| groups.remove(&s)).collect()
}

/// A flat, recursive listing of everything under `prefix`, collapsed by stem.
/// Clicking an entry queues a `comfyui/… -> models/…` copy for every file in it.
/// Reused for both columns — only `title` / `prefix` differ.
#[component]
fn ModelList(
    api: PlapApi,
    title: &'static str,
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
            <header class="panel-head">
                <h2 class="panel-title">{title}</h2>
                <span class="panel-prefix">{prefix}</span>
            </header>

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
                                    let groups = collapse(prefix, resp.tree.paths);
                                    if groups.is_empty() {
                                        return view! {
                                            <p class="muted panel-note">"Nothing here."</p>
                                        }
                                            .into_any();
                                    }
                                    let rows = groups
                                        .into_iter()
                                        .map(|g| {
                                            let api = api.clone();
                                            let bucket = bucket.clone();
                                            let Group { name, exts, keys } = g;
                                            let label = name.clone();
                                            let n = keys.len();
                                            let on_click = move |_| {
                                                let api = api.clone();
                                                let bucket = bucket.clone();
                                                let keys = keys.clone();
                                                let label = label.clone();
                                                set_status
                                                    .set(format!("Queuing {label} ({n})…"));
                                                spawn_local(async move {
                                                    let mut ok = 0usize;
                                                    let mut failed: Option<String> = None;
                                                    for k in &keys {
                                                        match api
                                                            .cp_model(
                                                                bucket.clone(),
                                                                StoragePath(k.clone()),
                                                            )
                                                            .await
                                                        {
                                                            Ok(_) => ok += 1,
                                                            Err(e) => {
                                                                failed = Some(e.to_string());
                                                                break;
                                                            }
                                                        }
                                                    }
                                                    set_status
                                                        .set(match failed {
                                                            None => {
                                                                format!(
                                                                    "Queued {label} — {ok} file(s)",
                                                                )
                                                            }
                                                            Some(e) => {
                                                                format!(
                                                                    "Failed {label} after {ok}: {e}",
                                                                )
                                                            }
                                                        });
                                                });
                                            };
                                            let tags = exts
                                                .into_iter()
                                                .map(|e| view! { <span class="tag">{e}</span> })
                                                .collect_view();
                                            view! {
                                                <li
                                                    class="item"
                                                    title="Promote to models/"
                                                    on:click=on_click
                                                >
                                                    <span class="item-name">{name}</span>
                                                    <span class="item-tags">{tags}</span>
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
                    <ModelList
                        api=api.clone()
                        title="Diffusion models"
                        prefix=DIFFUSION_MODELS
                        set_session
                    />
                    <ModelList api=api.clone() title="LoRAs" prefix=LORAS set_session />
                </div>
            </section>
        </main>
    }
}
