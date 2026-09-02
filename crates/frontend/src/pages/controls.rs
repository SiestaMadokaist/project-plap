use std::collections::HashMap;

use domain::{
    errors::DomainError,
    storage::{StoragePath, StoragePrefix},
};
use leptos::prelude::*;
use pkg::{auth::claims::JWT, types::strings::URL};
use wasm_bindgen::JsValue;
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

/// What the preview modal is currently showing.
#[derive(Clone)]
struct PreviewReq {
    name: String,
    image: Option<StoragePath>,
    json: Option<StoragePath>,
}

/// Split a key into `(stem, ext)`. `.civitai.json` counts as one extension so a
/// civitai sidecar collapses into the same group as its model / `model.json`.
fn split_ext(key: &str) -> (&str, Option<String>) {
    match key.rsplit_once('.') {
        Some((rest, "json")) if rest.ends_with(".civitai") => {
            (&rest[..rest.len() - ".civitai".len()], Some("civitai.json".into()))
        }
        Some((stem, ext)) => (stem, Some(ext.to_ascii_lowercase())),
        None => (key, None),
    }
}

fn basename(key: &str) -> &str {
    key.rsplit_once('/').map(|(_, b)| b).unwrap_or(key)
}

/// Collapse a flat key listing into per-stem groups.
fn collapse(prefix: &str, paths: Vec<StoragePath>) -> Vec<Group> {
    let mut order: Vec<String> = Vec::new();
    let mut groups: HashMap<String, Group> = HashMap::new();

    for p in paths {
        let key = p.0;
        let (stem, ext) = split_ext(&key);
        let stem = stem.to_string();

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

/// Re-indent JSON to two spaces via the browser's own `JSON`. Falls back to the
/// raw text if it doesn't parse.
fn pretty_json(raw: &str) -> String {
    let parsed = match js_sys::JSON::parse(raw) {
        Ok(v) => v,
        Err(_) => return raw.to_string(),
    };
    js_sys::JSON::stringify_with_replacer_and_space(&parsed, &JsValue::NULL, &JsValue::from_f64(2.0))
        .ok()
        .and_then(|s| s.as_string())
        .unwrap_or_else(|| raw.to_string())
}

/// A flat, recursive listing of everything under `prefix`, collapsed by stem.
/// Clicking an entry queues a `comfyui/… -> models/…` copy for every file in it;
/// the eye button opens a preview of the image / json siblings.
/// Reused for both columns — only `title` / `prefix` differ.
#[component]
fn ModelList(
    api: PlapApi,
    title: &'static str,
    prefix: &'static str,
    set_session: WriteSignal<Option<JWT>>,
    /// bumped after a queue action so the agent-queue panel refetches
    bump_queue: WriteSignal<u32>,
) -> impl IntoView {
    let (status, set_status) = signal(String::new());
    let (preview, set_preview) = signal::<Option<PreviewReq>>(None);

    let items = LocalResource::new({
        let api = api.clone();
        move || {
            let api = api.clone();
            async move { api.list_models(StoragePrefix(prefix.into()), true).await }
        }
    });

    let preview_data = LocalResource::new({
        let api = api.clone();
        move || {
            let api = api.clone();
            let req = preview.get();
            async move {
                match req {
                    None => Ok::<_, DomainError>(None),
                    Some(r) => api.preview(r.image, r.json).await.map(Some),
                }
            }
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
                                            let n = keys.len();
                                            let img_key = keys
                                                .iter()
                                                .find(|k| {
                                                    matches!(
                                                        split_ext(k).1.as_deref(),
                                                        Some("png" | "jpg" | "jpeg")
                                                    )
                                                })
                                                .cloned();
                                            // prefer a plain `model.json`; fall back to the
                                            // civitai sidecar.
                                            let json_key = keys
                                                .iter()
                                                .find(|k| split_ext(k).1.as_deref() == Some("json"))
                                                .or_else(|| {
                                                    keys.iter().find(|k| {
                                                        split_ext(k).1.as_deref()
                                                            == Some("civitai.json")
                                                    })
                                                })
                                                .cloned();
                                            let has_preview =
                                                img_key.is_some() || json_key.is_some();

                                            let q_label = name.clone();
                                            let on_click = {
                                                let api = api.clone();
                                                let bucket = bucket.clone();
                                                move |_| {
                                                    let api = api.clone();
                                                    let bucket = bucket.clone();
                                                    let keys = keys.clone();
                                                    let label = q_label.clone();
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
                                                                None => format!(
                                                                    "Queued {label} — {ok} file(s)",
                                                                ),
                                                                Some(e) => format!(
                                                                    "Failed {label} after {ok}: {e}",
                                                                ),
                                                            });
                                                        if ok > 0 {
                                                            bump_queue.update(|n| *n += 1);
                                                        }
                                                    });
                                                }
                                            };

                                            let p_name = name.clone();
                                            let on_preview = move |ev: leptos::ev::MouseEvent| {
                                                ev.stop_propagation();
                                                set_preview
                                                    .set(Some(PreviewReq {
                                                        name: p_name.clone(),
                                                        image: img_key.clone().map(StoragePath),
                                                        json: json_key.clone().map(StoragePath),
                                                    }));
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
                                                    <div class="item-foot">
                                                        <span class="item-tags">{tags}</span>
                                                        {has_preview
                                                            .then(|| {
                                                                view! {
                                                                    <button
                                                                        class="peek-btn"
                                                                        title="Preview"
                                                                        on:click=on_preview
                                                                    >
                                                                        "👁"
                                                                    </button>
                                                                }
                                                            })}
                                                    </div>
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

            {move || {
                preview
                    .get()
                    .map(|req| {
                        let name = req.name.clone();
                        let image_name = req.image.as_ref().map(|p| basename(&p.0).to_string());
                        let json_name = req.json.as_ref().map(|p| basename(&p.0).to_string());
                        view! {
                            <div
                                class="modal-scrim"
                                on:click=move |_| set_preview.set(None)
                            >
                                <div
                                    class="modal"
                                    on:click=|ev: leptos::ev::MouseEvent| ev.stop_propagation()
                                >
                                    <header class="modal-head">
                                        <span class="modal-title">{name}</span>
                                        <button
                                            class="ghost-btn"
                                            on:click=move |_| set_preview.set(None)
                                        >
                                            "Close"
                                        </button>
                                    </header>
                                    <div class="modal-body">
                                        <Suspense fallback=|| {
                                            view! { <p class="muted">"Loading…"</p> }
                                        }>
                                            {
                                                let image_name = image_name.clone();
                                                let json_name = json_name.clone();
                                                move || {
                                                let image_name = image_name.clone();
                                                let json_name = json_name.clone();
                                                Suspend::new(async move {
                                                match preview_data.await {
                                                    Ok(Some(p)) => {
                                                        let img = p
                                                            .image_url
                                                            .map(|u| {
                                                                view! {
                                                                    <figure class="preview-fig">
                                                                        {image_name
                                                                            .map(|n| {
                                                                                view! {
                                                                                    <figcaption class="preview-cap">
                                                                                        {n}
                                                                                    </figcaption>
                                                                                }
                                                                            })}
                                                                        <img class="preview-img" src=u />
                                                                    </figure>
                                                                }
                                                            });
                                                        let js = p
                                                            .json
                                                            .map(|raw| {
                                                                view! {
                                                                    <figure class="preview-fig">
                                                                        {json_name
                                                                            .map(|n| {
                                                                                view! {
                                                                                    <figcaption class="preview-cap">
                                                                                        {n}
                                                                                    </figcaption>
                                                                                }
                                                                            })}
                                                                        <pre class="preview-json">
                                                                            {pretty_json(&raw)}
                                                                        </pre>
                                                                    </figure>
                                                                }
                                                            });
                                                        let empty = img.is_none() && js.is_none();
                                                        view! {
                                                            <div class="preview-stack">
                                                                {img}
                                                                {js}
                                                                {empty
                                                                    .then(|| {
                                                                        view! {
                                                                            <p class="muted">
                                                                                "No preview available."
                                                                            </p>
                                                                        }
                                                                    })}
                                                            </div>
                                                        }
                                                            .into_any()
                                                    }
                                                    Ok(None) => {
                                                        view! {
                                                            <p class="muted">"No preview available."</p>
                                                        }
                                                            .into_any()
                                                    }
                                                    Err(e) => {
                                                        view! {
                                                            <p class="auth-error">
                                                                {format!("Preview failed: {e}")}
                                                            </p>
                                                        }
                                                            .into_any()
                                                    }
                                                }
                                                })
                                                }
                                            }
                                        </Suspense>
                                    </div>
                                </div>
                            </div>
                        }
                    })
            }}
        </section>
    }
}

/// Agent commands currently in the queue. For now just `action_id` + `priority`,
/// each with a delete button.
#[component]
fn AgentCommands(
    api: PlapApi,
    set_session: WriteSignal<Option<JWT>>,
    /// refetch trigger — bumped here on delete, and by the model panels on queue
    reload: ReadSignal<u32>,
    set_reload: WriteSignal<u32>,
) -> impl IntoView {
    let (status, set_status) = signal(String::new());

    let items = LocalResource::new({
        let api = api.clone();
        move || {
            let api = api.clone();
            reload.track();
            async move { api.list_commands().await }
        }
    });

    view! {
        <section class="panel">
            <header class="panel-head">
                <h2 class="panel-title">"In progress"</h2>
                <span class="panel-prefix">"agent queue"</span>
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
                                    let cmds = resp.commands;
                                    if cmds.is_empty() {
                                        return view! {
                                            <p class="muted panel-note">"Nothing in progress."</p>
                                        }
                                            .into_any();
                                    }
                                    let rows = cmds
                                        .into_iter()
                                        .map(|c| {
                                            let id = c.action_id.0.clone();
                                            let prio = c.priority;
                                            let stage = c.status();
                                            let on_delete = {
                                                let api = api.clone();
                                                let del = c.action_id.clone();
                                                move |_| {
                                                    let api = api.clone();
                                                    let del = del.clone();
                                                    set_status
                                                        .set(format!("Deleting {}…", del.0));
                                                    spawn_local(async move {
                                                        match api.delete_command(del).await {
                                                            Ok(_) => set_status.set(String::new()),
                                                            Err(e) => set_status
                                                                .set(format!("Delete failed: {e}")),
                                                        }
                                                        set_reload.update(|n| *n += 1);
                                                    });
                                                }
                                            };
                                            view! {
                                                <li class="cmd">
                                                    <div class="cmd-main">
                                                        <span class="cmd-id">{id}</span>
                                                        <span class="cmd-prio">
                                                            <span class="cmd-stage">{stage}</span>
                                                            {format!(" · priority {prio}")}
                                                        </span>
                                                    </div>
                                                    <button
                                                        class="peek-btn cmd-del"
                                                        title="Delete"
                                                        on:click=on_delete
                                                    >
                                                        "✕"
                                                    </button>
                                                </li>
                                            }
                                        })
                                        .collect_view();
                                    view! { <ul class="item-list cmd-list">{rows}</ul> }.into_any()
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

    // shared refetch trigger for the agent-queue panel: queue actions in the model
    // panels and deletes in the queue panel all bump it.
    let (queue_reload, set_queue_reload) = signal(0u32);

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
                        bump_queue=set_queue_reload
                    />
                    <ModelList
                        api=api.clone()
                        title="LoRAs"
                        prefix=LORAS
                        set_session
                        bump_queue=set_queue_reload
                    />
                    <AgentCommands
                        api=api.clone()
                        set_session
                        reload=queue_reload
                        set_reload=set_queue_reload
                    />
                </div>
            </section>
        </main>
    }
}
