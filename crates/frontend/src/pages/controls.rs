use std::collections::HashMap;

use domain::{
    commands::compute::{ComputeArgs, ComputeCommand, ComputeRegion},
    errors::DomainError,
    storage::{StorageBucket, StoragePath, StoragePrefix},
};
use leptos::prelude::*;
use pkg::{auth::claims::JWT, types::strings::URL};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;

use crate::{api::plap::PlapApi, env::ENV, session};

/// Every region an EC2 engine may be configured for — see
/// `domain::commands::compute::ComputeRegion`. No enumeration helper exists on
/// the domain type itself, so the panel's region picker lists them by hand.
const COMPUTE_REGIONS: [ComputeRegion; 1] = [
    // ComputeRegion::AwsApSoutheast1,
    // ComputeRegion::AWSApSoutheast2,
    // ComputeRegion::AWSApSoutheast3,
    ComputeRegion::AWSUsEast1,
];

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
    /// image sample siblings, capped at [`MAX_PREVIEW_IMAGES`]
    images: Vec<StoragePath>,
    json: Option<StoragePath>,
}

/// How many image samples the preview modal shows for one entry.
const MAX_PREVIEW_IMAGES: usize = 3;

/// Split a key into `(stem, ext)`. Two shapes fold onto a shorter stem so they
/// group with the model they belong to rather than forming their own entry:
/// - `<stem>.civitai.json` — the civitai sidecar
/// - `<stem>.image-<n>.png` — a generated preview sample
fn split_ext(key: &str) -> (&str, Option<String>) {
    match key.rsplit_once('.') {
        Some((rest, "json")) if rest.ends_with(".civitai") => (
            &rest[..rest.len() - ".civitai".len()],
            Some("civitai.json".into()),
        ),
        Some((rest, "png")) => match sample_stem(rest) {
            Some(stem) => (stem, Some("png".into())),
            None => (rest, Some("png".into())),
        },
        Some((stem, ext)) => (stem, Some(ext.to_ascii_lowercase())),
        None => (key, None),
    }
}

/// `foo/bar.image-3` -> `Some("foo/bar")`; the segment after `.image-` must be
/// all digits, otherwise this isn't a preview sample.
fn sample_stem(rest: &str) -> Option<&str> {
    let (stem, n) = rest.rsplit_once(".image-")?;
    (!n.is_empty() && n.bytes().all(|b| b.is_ascii_digit())).then_some(stem)
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
    order
        .into_iter()
        .filter_map(|s| groups.remove(&s))
        .collect()
}

/// Re-indent JSON to two spaces via the browser's own `JSON`. Falls back to the
/// raw text if it doesn't parse.
fn pretty_json(raw: &str) -> String {
    let parsed = match js_sys::JSON::parse(raw) {
        Ok(v) => v,
        Err(_) => return raw.to_string(),
    };
    js_sys::JSON::stringify_with_replacer_and_space(
        &parsed,
        &JsValue::NULL,
        &JsValue::from_f64(2.0),
    )
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
    // bumped by the panel's refresh button to re-run the listing
    let (reload, set_reload) = signal(0u32);

    let items = LocalResource::new({
        let api = api.clone();
        move || {
            let api = api.clone();
            reload.track();
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
                    Some(r) => api.preview(r.images, r.json).await.map(Some),
                }
            }
        }
    });

    view! {
        <section class="panel">
            <header class="panel-head">
                <h2 class="panel-title">{title}</h2>
                <span class="panel-prefix">{prefix}</span>
                <button
                    class="peek-btn panel-refresh"
                    title="Refresh"
                    on:click=move |_| set_reload.update(|n| *n += 1)
                >
                    "↻"
                </button>
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
                                            let Group { name, exts, keys } = g;
                                            let n = keys.len();
                                            // S3 lists lexicographically, so `.image-1/2/3.png`
                                            // sort ahead of a `.jpeg` — take the first few.
                                            let img_keys: Vec<String> = keys
                                                .iter()
                                                .filter(|k| {
                                                    matches!(
                                                        split_ext(k).1.as_deref(),
                                                        Some("png" | "jpg" | "jpeg")
                                                    )
                                                })
                                                .take(MAX_PREVIEW_IMAGES)
                                                .cloned()
                                                .collect();
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
                                                !img_keys.is_empty() || json_key.is_some();

                                            let q_label = name.clone();
                                            let on_click = {
                                                let api = api.clone();
                                                move |_| {
                                                    let api = api.clone();
                                                    let keys = keys.clone();
                                                    let label = q_label.clone();
                                                    set_status
                                                        .set(format!("Queuing {label} ({n})…"));
                                                    spawn_local(async move {
                                                        let mut ok = 0usize;
                                                        let mut failed: Option<String> = None;
                                                        for k in &keys {
                                                            match api
                                                                .cp_model(StoragePath(k.clone()))
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
                                                        images: img_keys
                                                            .iter()
                                                            .cloned()
                                                            .map(StoragePath)
                                                            .collect(),
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
                        let image_names: Vec<String> =
                            req.images.iter().map(|p| basename(&p.0).to_string()).collect();
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
                                                let image_names = image_names.clone();
                                                let json_name = json_name.clone();
                                                move || {
                                                let image_names = image_names.clone();
                                                let json_name = json_name.clone();
                                                Suspend::new(async move {
                                                match preview_data.await {
                                                    Ok(Some(p)) => {
                                                        let has_img = !p.image_urls.is_empty();
                                                        let has_json = p.json.is_some();
                                                        // one <figure> per sample, laid out in a
                                                        // fixed 3-col grid (a lone image stays 1/3 wide)
                                                        let imgs = has_img.then(|| {
                                                            let figs = p
                                                                .image_urls
                                                                .into_iter()
                                                                .zip(image_names)
                                                                .map(|(u, n)| {
                                                                    view! {
                                                                        <figure class="preview-fig">
                                                                            <figcaption class="preview-cap">
                                                                                {n}
                                                                            </figcaption>
                                                                            <img class="preview-img" src=u />
                                                                        </figure>
                                                                    }
                                                                })
                                                                .collect_view();
                                                            view! { <div class="preview-images">{figs}</div> }
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
                                                        view! {
                                                            <div class="preview-stack">
                                                                {imgs}
                                                                {js}
                                                                {(!has_img && !has_json)
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

/// Standalone action panel: queue the `comfyui/bootstraps/ -> models/` copy.
/// Sits above the "Diffusion models" list in its column.
#[component]
fn SyncBootstraps(api: PlapApi, bump_queue: WriteSignal<u32>) -> impl IntoView {
    let (status, set_status) = signal(String::new());

    let on_sync = move |_| {
        let api = api.clone();
        set_status.set("Queuing bootstraps…".to_string());
        spawn_local(async move {
            match api.cp_bootstraps().await {
                Ok(_) => {
                    set_status.set("Queued comfyui/bootstraps/ -> models/".to_string());
                    bump_queue.update(|n| *n += 1);
                }
                Err(e) => set_status.set(format!("Bootstrap sync failed: {e}")),
            }
        });
    };

    view! {
        <section class="panel">
            <header class="panel-head">
                <h2 class="panel-title">"Bootstraps"</h2>
                <span class="panel-prefix">"comfyui/bootstraps/ -> models/"</span>
            </header>

            {move || {
                let s = status.get();
                (!s.is_empty()).then(|| view! { <p class="panel-status">{s}</p> })
            }}

            <div class="panel-actions">
                <button class="ghost-btn" on:click=on_sync>
                    "Sync bootstraps -> models/"
                </button>
            </div>
        </section>
    }
}

/// Standalone action panel: pull a civitai model version by id into the agent
/// queue. Sits above the "In progress" queue in its column.
#[component]
fn CivitaiPull(api: PlapApi, bump_queue: WriteSignal<u32>) -> impl IntoView {
    let (status, set_status) = signal(String::new());
    let (version_id, set_version_id) = signal(String::new());

    // Enter queues a civitai -> localhost download for that model version id.
    let on_key = move |ev: leptos::ev::KeyboardEvent| {
        if ev.key() != "Enter" {
            return;
        }
        let raw = version_id.get_untracked();
        let trimmed = raw.trim();
        let id: u32 = match trimmed.parse() {
            Ok(id) if id > 0 => id,
            _ => {
                set_status.set(format!("'{trimmed}' is not a valid civitai version id"));
                return;
            }
        };
        set_status.set(format!("Queuing civitai {id}…"));
        set_version_id.set(String::new());
        let api = api.clone();
        spawn_local(async move {
            match api.cp_civitai(id).await {
                Ok(_) => {
                    set_status.set(format!("Queued civitai {id}"));
                    bump_queue.update(|n| *n += 1);
                }
                Err(e) => set_status.set(format!("civitai {id} failed: {e}")),
            }
        });
    };

    view! {
        <section class="panel">
            <header class="panel-head">
                <h2 class="panel-title">"Civitai"</h2>
                <span class="panel-prefix">"version id -> queue"</span>
            </header>

            {move || {
                let s = status.get();
                (!s.is_empty()).then(|| view! { <p class="panel-status">{s}</p> })
            }}

            <div class="panel-actions">
                <input
                    class="panel-input"
                    type="number"
                    min="1"
                    placeholder="civitai version id — press Enter"
                    prop:value=move || version_id.get()
                    on:input=move |ev| set_version_id.set(event_target_value(&ev))
                    on:keydown=on_key
                />
            </div>
        </section>
    }
}

/// Agent commands currently in the queue. For now just `action_id` + `priority`,
/// each with a delete button.
#[component]
fn AgentCommands(
    api: PlapApi,
    set_session: WriteSignal<Option<JWT>>,
    /// refetch trigger — bumped here on delete, and by the action panels on queue
    reload: ReadSignal<u32>,
    set_reload: WriteSignal<u32>,
) -> impl IntoView {
    let (status, set_status) = signal(String::new());

    let items = LocalResource::new({
        let api = api.clone();
        move || {
            let api = api.clone();
            reload.track();
            async move { api.list_taskqueue().await }
        }
    });

    view! {
        <section class="panel">
            <header class="panel-head">
                <h2 class="panel-title">"In progress"</h2>
                <span class="panel-prefix">"agent queue"</span>
                <button
                    class="peek-btn panel-refresh"
                    title="Refresh"
                    on:click=move |_| set_reload.update(|n| *n += 1)
                >
                    "↻"
                </button>
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

/// Launch/list/start/stop/reboot panel for the EC2 instances behind `hq`.
/// Listing, launch, and per-instance actions are all scoped to whichever
/// region is currently selected in the picker.
#[component]
fn ComputePanel(api: PlapApi, set_session: WriteSignal<Option<JWT>>) -> impl IntoView {
    let (status, set_status) = signal(String::new());
    let (region, set_region) = signal(COMPUTE_REGIONS[0]);
    let (reload, set_reload) = signal(0u32);

    let items = LocalResource::new({
        let api = api.clone();
        move || {
            let api = api.clone();
            let region = region.get();
            reload.track();
            async move { api.list_compute(region).await }
        }
    });

    let on_region_change = move |ev: leptos::ev::Event| {
        let raw = event_target_value(&ev);
        if let Ok(r) = ComputeRegion::try_from(raw.as_str()) {
            set_region.set(r);
        }
    };

    // shared by the "Launch on demand" / "Launch spot instance" buttons — uses
    // whichever region is currently selected in the picker above.
    let launch = {
        let api = api.clone();
        move |spot: bool| {
            let api = api.clone();
            let region = region.get_untracked();
            let kind = if spot { "spot" } else { "on-demand" };
            set_status.set(format!("Launching {kind} instance in {region}…"));
            spawn_local(async move {
                match api.launch_compute(spot, region).await {
                    Ok(dto) => set_status.set(format!("Launched {kind} {}", dto.0.id)),
                    Err(e) => set_status.set(format!("Launch failed: {e}")),
                }
                set_reload.update(|n| *n += 1);
            });
        }
    };
    let on_launch_on_demand = {
        let launch = launch.clone();
        move |_| launch(false)
    };
    let on_launch_spot = move |_| launch(true);

    // shared by the Start/Stop/Reboot buttons on every row — only the command differs.
    let dispatch = {
        let api = api.clone();
        move |id: domain::commands::compute::ComputeInstanceID, command: ComputeCommand| {
            let api = api.clone();
            let region = region.get_untracked();
            let label = format!("{command:?}");
            set_status.set(format!("{label} {id}…"));
            spawn_local(async move {
                let args = ComputeArgs {
                    instance_id: id.clone(),
                    command,
                    region,
                };
                match api.control_compute(args).await {
                    Ok(_) => set_status.set(format!("{label} sent to {id}")),
                    Err(e) => set_status.set(format!("{label} on {id} failed: {e}")),
                }
                set_reload.update(|n| *n += 1);
            });
        }
    };

    view! {
        <section class="panel compute-panel">
            <header class="panel-head">
                <h2 class="panel-title">"Compute"</h2>
                <select class="panel-input compute-region" on:change=on_region_change>
                    {COMPUTE_REGIONS
                        .iter()
                        .map(|r| {
                            let value = r.to_string();
                            let r = *r;
                            view! {
                                <option value=value.clone() selected=move || region.get() == r>
                                    {value.clone()}
                                </option>
                            }
                        })
                        .collect_view()}
                </select>
                <button class="ghost-btn" on:click=on_launch_on_demand>
                    "Launch on demand"
                </button>
                <button class="ghost-btn" on:click=on_launch_spot>
                    "Launch spot instance"
                </button>
                <button
                    class="peek-btn panel-refresh"
                    title="Refresh"
                    on:click=move |_| set_reload.update(|n| *n += 1)
                >
                    "↻"
                </button>
            </header>

            {move || {
                let s = status.get();
                (!s.is_empty()).then(|| view! { <p class="panel-status">{s}</p> })
            }}

            <Suspense fallback=|| {
                view! { <p class="muted panel-note">"Loading…"</p> }
            }>
                {
                    let dispatch = dispatch.clone();
                    move || {
                        let dispatch = dispatch.clone();
                        Suspend::new(async move {
                            match items.await {
                                Ok(resp) => {
                                    if resp.instances.is_empty() {
                                        return view! {
                                            <p class="muted panel-note">
                                                "No instances in this region."
                                            </p>
                                        }
                                            .into_any();
                                    }
                                    let rows = resp
                                        .instances
                                        .into_iter()
                                        .map(|inst| {
                                            let id = inst.id.clone();
                                            let ip = inst
                                                .ip
                                                .map(|ip| ip.to_string())
                                                .unwrap_or_else(|| "-".to_string());
                                            let meta = format!(
                                                "{} · {}{} · {ip}",
                                                inst.status,
                                                inst.tipe,
                                                if inst.is_spot { " · spot" } else { "" },
                                            );
                                            let start = {
                                                let dispatch = dispatch.clone();
                                                let id = id.clone();
                                                move |_| dispatch(id.clone(), ComputeCommand::Start)
                                            };
                                            let stop = {
                                                let dispatch = dispatch.clone();
                                                let id = id.clone();
                                                move |_| dispatch(id.clone(), ComputeCommand::Stop)
                                            };
                                            let reboot = {
                                                let dispatch = dispatch.clone();
                                                let id = id.clone();
                                                move |_| dispatch(id.clone(), ComputeCommand::Reboot)
                                            };
                                            let terminate = {
                                                let dispatch = dispatch.clone();
                                                let id = id.clone();
                                                move |_| {
                                                    dispatch(id.clone(), ComputeCommand::Terminate)
                                                }
                                            };
                                            view! {
                                                <li class="cmd">
                                                    <div class="cmd-main">
                                                        <span class="cmd-id">{id.to_string()}</span>
                                                        <span class="cmd-prio">{meta}</span>
                                                    </div>
                                                    <div class="cmd-actions">
                                                        <button class="peek-btn" on:click=start>
                                                            "Start"
                                                        </button>
                                                        <button class="peek-btn" on:click=stop>
                                                            "Stop"
                                                        </button>
                                                        <button class="peek-btn" on:click=reboot>
                                                            "Reboot"
                                                        </button>
                                                        <button
                                                            class="peek-btn danger"
                                                            on:click=terminate
                                                        >
                                                            "Terminate"
                                                        </button>
                                                    </div>
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
    let api = PlapApi::new(
        jwt,
        URL(ENV.api_base.to_string()),
        StorageBucket(ENV.model_bucket.to_string()),
        StorageBucket(ENV.io_bucket.to_string()),
    );

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
                <ComputePanel api=api.clone() set_session />
                <div class="panels">
                    <div class="panel-col">
                        <SyncBootstraps api=api.clone() bump_queue=set_queue_reload />
                        <ModelList
                            api=api.clone()
                            title="Diffusion models"
                            prefix=DIFFUSION_MODELS
                            set_session
                            bump_queue=set_queue_reload
                        />
                    </div>
                    <div class="panel-col">
                        <ModelList
                            api=api.clone()
                            title="LoRAs"
                            prefix=LORAS
                            set_session
                            bump_queue=set_queue_reload
                        />
                    </div>
                    <div class="panel-col">
                        <CivitaiPull api=api.clone() bump_queue=set_queue_reload />
                        <AgentCommands
                            api=api.clone()
                            set_session
                            reload=queue_reload
                            set_reload=set_queue_reload
                        />
                    </div>
                </div>
            </section>
        </main>
    }
}
