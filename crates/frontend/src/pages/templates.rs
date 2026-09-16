use std::collections::HashMap;

use domain::{
    commands::inference::InferenceConfig,
    errors::DomainError,
    storage::StoragePath,
    storyline::{RawTemplate, Storyline, StoryTemplateId},
};
use leptos::prelude::*;
use pkg::{
    auth::claims::JWT,
    types::{strings::URL, unit},
};
use wasm_bindgen_futures::spawn_local;

use crate::{api::template::TemplateApi, env::ENV, session};

/// Substitute every `{{token}}` in `raw` with `vars[token]`. A token with no
/// matching variable is left in place (trimmed) so a missing mapping is
/// obvious in the compiled preview rather than silently vanishing.
fn compile(raw: &str, vars: &HashMap<String, String>) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        match after.find("}}") {
            Some(end) => {
                let key = after[..end].trim();
                match vars.get(key) {
                    Some(v) => out.push_str(v),
                    None => {
                        out.push_str("{{");
                        out.push_str(key);
                        out.push_str("}}");
                    }
                }
                rest = &after[end + 2..];
            }
            None => {
                out.push_str("{{");
                rest = after;
                break;
            }
        }
    }
    out.push_str(rest);
    out
}

/// Every distinct `{{token}}` referenced in `raw`, in first-seen order.
fn tokens_in(raw: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = raw;
    while let Some(start) = rest.find("{{") {
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else { break };
        let key = after[..end].trim().to_string();
        if !found.contains(&key) {
            found.push(key);
        }
        rest = &after[end + 2..];
    }
    found
}

/// A brand new inference slot: no checkpoint/workflow picked yet, common
/// stable-diffusion-ish defaults for the run settings.
fn default_inference() -> InferenceConfig<RawTemplate> {
    InferenceConfig {
        workflow_id: None,
        loras: Vec::new(),
        checkpoint: String::new(),
        positive: RawTemplate(String::new()),
        negative: RawTemplate(String::new()),
        width: unit::Px(512),
        height: unit::Px(512),
        steps: unit::Index1(20),
        n_iter: unit::Index1(1),
        seed: -1,
        references: Vec::new(),
    }
}

/// A brand new, empty template: one blank inference, no variables.
fn blank_storyline(id: String) -> Storyline {
    Storyline {
        id: StoryTemplateId(id),
        inferences: vec![default_inference()],
        variables: HashMap::new(),
    }
}

/// Either an existing template to load by id, or a freshly-created one that
/// hasn't been saved yet.
#[derive(Clone)]
enum TemplateTarget {
    Existing(String),
    New(Storyline),
}

/// One row of the variables editor: a fixed key plus an editable value.
#[derive(Clone)]
struct VariableEntry {
    key: String,
    value: RwSignal<String>,
}

/// Union of `storyline.variables` and every token referenced by an
/// inference's `positive` / `negative` template — stored variables first
/// (alphabetically, since `HashMap` has no stable order of its own), then
/// any template reference that isn't defined yet (seeded with an empty
/// value so it shows up as needing one).
fn collect_variables(storyline: &Storyline) -> Vec<VariableEntry> {
    let mut order: Vec<String> = storyline.variables.keys().cloned().collect();
    order.sort();
    let mut values = storyline.variables.clone();

    for inf in &storyline.inferences {
        for text in [&inf.positive.0, &inf.negative.0] {
            for token in tokens_in(text) {
                values.entry(token.clone()).or_insert_with(|| {
                    order.push(token);
                    String::new()
                });
            }
        }
    }

    order
        .into_iter()
        .map(|key| {
            let value = values.remove(&key).unwrap_or_default();
            VariableEntry {
                key,
                value: RwSignal::new(value),
            }
        })
        .collect()
}

/// One inference slot, fully editable: run settings plus the two raw
/// templates. `references` isn't exposed in the UI (it's an obscure,
/// rarely-touched field) but is carried through unedited on save.
#[derive(Clone)]
struct InferenceRowData {
    /// stable identity for this row — not a position, since rows can be
    /// added/removed
    key: u32,
    workflow_id: RwSignal<String>,
    checkpoint: RwSignal<String>,
    /// one lora path per line
    loras: RwSignal<String>,
    width: RwSignal<i32>,
    height: RwSignal<i32>,
    steps: RwSignal<i32>,
    n_iter: RwSignal<i32>,
    seed: RwSignal<i32>,
    positive: RwSignal<String>,
    negative: RwSignal<String>,
    references: Vec<String>,
}

fn to_row(base: InferenceConfig<RawTemplate>, key: u32) -> InferenceRowData {
    InferenceRowData {
        key,
        workflow_id: RwSignal::new(base.workflow_id.map(|w| w.0).unwrap_or_default()),
        checkpoint: RwSignal::new(base.checkpoint),
        loras: RwSignal::new(base.loras.into_iter().map(|l| l.0).collect::<Vec<_>>().join("\n")),
        width: RwSignal::new(base.width.0),
        height: RwSignal::new(base.height.0),
        steps: RwSignal::new(base.steps.0),
        n_iter: RwSignal::new(base.n_iter.0),
        seed: RwSignal::new(base.seed),
        positive: RwSignal::new(base.positive.0),
        negative: RwSignal::new(base.negative.0),
        references: base.references.into_iter().map(|u| u.0).collect(),
    }
}

fn to_inference(row: &InferenceRowData) -> InferenceConfig<RawTemplate> {
    let workflow_id = row.workflow_id.get_untracked();
    let workflow_id = (!workflow_id.trim().is_empty())
        .then(|| StoragePath(workflow_id.trim().to_string()));
    let loras = row
        .loras
        .get_untracked()
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| StoragePath(s.to_string()))
        .collect();

    InferenceConfig {
        workflow_id,
        loras,
        checkpoint: row.checkpoint.get_untracked(),
        positive: RawTemplate(row.positive.get_untracked()),
        negative: RawTemplate(row.negative.get_untracked()),
        width: unit::Px(row.width.get_untracked()),
        height: unit::Px(row.height.get_untracked()),
        steps: unit::Index1(row.steps.get_untracked()),
        n_iter: unit::Index1(row.n_iter.get_untracked()),
        seed: row.seed.get_untracked(),
        references: row.references.iter().cloned().map(URL).collect(),
    }
}

fn collect_inferences(storyline: &Storyline) -> Vec<InferenceRowData> {
    storyline
        .inferences
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, inf)| to_row(inf, i as u32))
        .collect()
}

/// Parse a `<input type=number>` edit; invalid/empty input is ignored and
/// the signal keeps its last value rather than snapping to zero.
fn set_from_input(sig: RwSignal<i32>, ev: leptos::ev::Event) {
    if let Ok(v) = event_target_value(&ev).parse::<i32>() {
        sig.set(v);
    }
}

/// One `<option>` in the template picker.
#[component]
fn TemplateOption(id: String, selected: Memo<Option<String>>) -> impl IntoView {
    let value = id.clone();
    let label = id.clone();
    view! {
        <option value=value.clone() selected=move || selected.get().as_deref() == Some(value.as_str())>
            {label}
        </option>
    }
}

/// Dropdown of every template id belonging to the caller.
#[component]
fn TemplateSelect(
    ids: ReadSignal<Vec<String>>,
    selected: Memo<Option<String>>,
    set_selected: WriteSignal<Option<TemplateTarget>>,
) -> impl IntoView {
    let on_change = move |ev: leptos::ev::Event| {
        let v = event_target_value(&ev);
        set_selected.set((!v.is_empty()).then_some(TemplateTarget::Existing(v)));
    };

    view! {
        <select class="panel-input tpl-select" on:change=on_change>
            <option value="" selected=move || selected.get().is_none()>
                "— choose a template —"
            </option>
            {move || {
                ids.get()
                    .into_iter()
                    .map(|id| view! { <TemplateOption id selected /> })
                    .collect_view()
            }}
        </select>
    }
}

/// Text field + button to start a brand new, blank template. Refuses an id
/// that's empty or already taken.
#[component]
fn NewTemplateForm(ids: ReadSignal<Vec<String>>, on_create: Callback<String>) -> impl IntoView {
    let (id, set_id) = signal(String::new());
    let (error, set_error) = signal(String::new());

    let submit = move |_| {
        let trimmed = id.get_untracked().trim().to_string();
        if trimmed.is_empty() {
            set_error.set("Template id can't be empty".to_string());
            return;
        }
        if ids.get_untracked().iter().any(|x| x == &trimmed) {
            set_error.set(format!("'{trimmed}' already exists — pick another id"));
            return;
        }
        set_error.set(String::new());
        set_id.set(String::new());
        on_create.run(trimmed);
    };

    view! {
        <div class="new-template">
            <input
                class="panel-input"
                placeholder="new template id"
                prop:value=move || id.get()
                on:input=move |ev| set_id.set(event_target_value(&ev))
            />
            <button class="ghost-btn" on:click=submit>
                "+ New template"
            </button>
            {move || {
                let e = error.get();
                (!e.is_empty()).then(|| view! { <span class="field-error">{e}</span> })
            }}
        </div>
    }
}

/// One editable variable: its key (fixed) and value (editable), with a
/// button to drop it from the set entirely.
#[component]
fn VariableRow(entry: VariableEntry, on_remove: Callback<String>) -> impl IntoView {
    let VariableEntry { key, value } = entry;
    let remove_key = key.clone();

    view! {
        <li class="cmd variable-row">
            <div class="cmd-main">
                <span class="cmd-id">{key}</span>
                <input
                    class="panel-input"
                    prop:value=move || value.get()
                    on:input=move |ev| value.set(event_target_value(&ev))
                />
            </div>
            <button
                class="peek-btn cmd-del"
                title="Remove"
                on:click=move |_| on_remove.run(remove_key.clone())
            >
                "✕"
            </button>
        </li>
    }
}

/// The full variable set for a template: existing + referenced-but-undefined
/// keys, each editable, plus a form to introduce a brand new one.
#[component]
fn VariablesPanel(variables: RwSignal<Vec<VariableEntry>>) -> impl IntoView {
    let (new_key, set_new_key) = signal(String::new());
    let (new_val, set_new_val) = signal(String::new());

    let on_remove = Callback::new(move |key: String| {
        variables.update(|v| v.retain(|e| e.key != key));
    });

    let on_add = move |_| {
        let key = new_key.get_untracked().trim().to_string();
        if key.is_empty() {
            return;
        }
        let val = new_val.get_untracked();
        variables.update(|v| match v.iter().find(|e| e.key == key) {
            Some(existing) => existing.value.set(val),
            None => v.push(VariableEntry {
                key,
                value: RwSignal::new(val),
            }),
        });
        set_new_key.set(String::new());
        set_new_val.set(String::new());
    };

    view! {
        <section class="panel">
            <header class="panel-head">
                <h2 class="panel-title">"Variables"</h2>
            </header>
            <ul class="item-list cmd-list">
                {move || {
                    variables
                        .get()
                        .into_iter()
                        .map(|entry| view! { <VariableRow entry on_remove /> })
                        .collect_view()
                }}
            </ul>
            <div class="panel-actions variable-add">
                <input
                    class="panel-input"
                    placeholder="key, e.g. mc:female"
                    prop:value=move || new_key.get()
                    on:input=move |ev| set_new_key.set(event_target_value(&ev))
                />
                <input
                    class="panel-input"
                    placeholder="value"
                    prop:value=move || new_val.get()
                    on:input=move |ev| set_new_val.set(event_target_value(&ev))
                />
                <button class="ghost-btn" on:click=on_add>
                    "Add variable"
                </button>
            </div>
        </section>
    }
}

/// One inference: its fully editable run settings, the two editable raw
/// templates, and a live-compiled preview of each against the current
/// variable set.
#[component]
fn InferenceRow(
    row: InferenceRowData,
    vars: Memo<HashMap<String, String>>,
    on_remove: Callback<u32>,
) -> impl IntoView {
    let InferenceRowData {
        key,
        workflow_id,
        checkpoint,
        loras,
        width,
        height,
        steps,
        n_iter,
        seed,
        positive,
        negative,
        references: _,
    } = row;

    let compiled_positive = Memo::new(move |_| compile(&positive.get(), &vars.get()));
    let compiled_negative = Memo::new(move |_| compile(&negative.get(), &vars.get()));

    view! {
        <li class="inference-row">
            <div class="inference-head">
                <span class="inference-idx">{format!("#{}", key + 1)}</span>
                <button
                    class="peek-btn danger"
                    title="Remove inference"
                    on:click=move |_| on_remove.run(key)
                >
                    "✕"
                </button>
            </div>

            <div class="inference-grid">
                <div class="field">
                    <span class="field-label">"Checkpoint"</span>
                    <input
                        class="panel-input"
                        prop:value=move || checkpoint.get()
                        on:input=move |ev| checkpoint.set(event_target_value(&ev))
                    />
                </div>
                <div class="field">
                    <span class="field-label">"Workflow id"</span>
                    <input
                        class="panel-input"
                        placeholder="(none)"
                        prop:value=move || workflow_id.get()
                        on:input=move |ev| workflow_id.set(event_target_value(&ev))
                    />
                </div>
                <div class="field">
                    <span class="field-label">"Width"</span>
                    <input
                        class="panel-input"
                        type="number"
                        min="1"
                        prop:value=move || width.get().to_string()
                        on:input=move |ev| set_from_input(width, ev)
                    />
                </div>
                <div class="field">
                    <span class="field-label">"Height"</span>
                    <input
                        class="panel-input"
                        type="number"
                        min="1"
                        prop:value=move || height.get().to_string()
                        on:input=move |ev| set_from_input(height, ev)
                    />
                </div>
                <div class="field">
                    <span class="field-label">"Steps"</span>
                    <input
                        class="panel-input"
                        type="number"
                        min="1"
                        prop:value=move || steps.get().to_string()
                        on:input=move |ev| set_from_input(steps, ev)
                    />
                </div>
                <div class="field">
                    <span class="field-label">"Iterations (max 4)"</span>
                    <input
                        class="panel-input"
                        type="number"
                        min="1"
                        max="4"
                        prop:value=move || n_iter.get().to_string()
                        on:input=move |ev| set_from_input(n_iter, ev)
                    />
                </div>
                <div class="field">
                    <span class="field-label">"Seed"</span>
                    <input
                        class="panel-input"
                        type="number"
                        prop:value=move || seed.get().to_string()
                        on:input=move |ev| set_from_input(seed, ev)
                    />
                </div>
            </div>

            <label class="field-label">"LoRAs (one path per line)"</label>
            <textarea
                class="panel-input tpl-textarea small"
                prop:value=move || loras.get()
                on:input=move |ev| loras.set(event_target_value(&ev))
            ></textarea>

            <label class="field-label">"Positive"</label>
            <textarea
                class="panel-input tpl-textarea"
                prop:value=move || positive.get()
                on:input=move |ev| positive.set(event_target_value(&ev))
            ></textarea>
            <pre class="preview-json tpl-compiled">{move || compiled_positive.get()}</pre>

            <label class="field-label">"Negative"</label>
            <textarea
                class="panel-input tpl-textarea"
                prop:value=move || negative.get()
                on:input=move |ev| negative.set(event_target_value(&ev))
            ></textarea>
            <pre class="preview-json tpl-compiled">{move || compiled_negative.get()}</pre>
        </li>
    }
}

/// Shared editor body for a `Storyline`, whether it was just fetched or was
/// just created blank in memory. All per-field signals are created here,
/// once, from the initial `storyline` — typing never re-fetches or rebuilds
/// unrelated rows.
#[component]
fn StorylineForm(
    api: TemplateApi,
    storyline: Storyline,
    set_session: WriteSignal<Option<JWT>>,
    on_saved: Callback<String>,
) -> impl IntoView {
    let id = storyline.id.0.clone();
    let (status, set_status) = signal(String::new());
    let variables = RwSignal::new(collect_variables(&storyline));
    let (inferences, set_inferences) = signal(collect_inferences(&storyline));
    let next_key = RwSignal::new(storyline.inferences.len() as u32);

    let vars_map = Memo::new(move |_| {
        variables
            .get()
            .into_iter()
            .map(|e| (e.key, e.value.get()))
            .collect::<HashMap<_, _>>()
    });

    let on_add_inference = move |_| {
        let key = next_key.get_untracked();
        next_key.set(key + 1);
        set_inferences.update(|rows| rows.push(to_row(default_inference(), key)));
    };

    let on_remove_inference = Callback::new(move |key: u32| {
        set_inferences.update(|rows| rows.retain(|r| r.key != key));
    });

    let on_save = {
        let api = api.clone();
        let id = id.clone();
        move |_| {
            let api = api.clone();
            let id = id.clone();
            let edited = Storyline {
                id: StoryTemplateId(id.clone()),
                inferences: inferences.get_untracked().iter().map(to_inference).collect(),
                variables: variables
                    .get_untracked()
                    .into_iter()
                    .map(|e| (e.key, e.value.get_untracked()))
                    .collect(),
            };

            set_status.set("Saving…".to_string());
            spawn_local(async move {
                match api.write(edited).await {
                    Ok(_) => {
                        set_status.set("Saved".to_string());
                        on_saved.run(id);
                    }
                    Err(DomainError::NotAllowed(_)) => {
                        session::clear();
                        set_session.set(None);
                    }
                    Err(e) => set_status.set(format!("Save failed: {e}")),
                }
            });
        }
    };

    view! {
        <div class="tpl-body">
            {move || {
                let s = status.get();
                (!s.is_empty()).then(|| view! { <p class="panel-status">{s}</p> })
            }}

            <VariablesPanel variables />

            <div class="tpl-inferences">
                <div class="tpl-inferences-head">
                    <h3 class="tpl-subtitle">"Inferences"</h3>
                    <button class="ghost-btn" on:click=on_add_inference>
                        "+ Add inference"
                    </button>
                </div>
                <ul class="item-list tpl-inference-list">
                    {move || {
                        inferences
                            .get()
                            .into_iter()
                            .map(|row| {
                                view! {
                                    <InferenceRow row vars=vars_map on_remove=on_remove_inference />
                                }
                            })
                            .collect_view()
                    }}
                </ul>
            </div>

            <div class="panel-actions">
                <button class="ghost-btn" on:click=on_save>
                    "Save changes"
                </button>
            </div>
        </div>
    }
}

/// Loads one existing template by id, then hands it to [`StorylineForm`].
#[component]
fn ExistingTemplateEditor(
    api: TemplateApi,
    id: String,
    set_session: WriteSignal<Option<JWT>>,
    on_saved: Callback<String>,
) -> impl IntoView {
    let (reload, set_reload) = signal(0u32);

    let data = LocalResource::new({
        let api = api.clone();
        let id = id.clone();
        move || {
            let api = api.clone();
            let id = id.clone();
            reload.track();
            async move { api.read(&id).await }
        }
    });

    view! {
        <section class="panel template-panel">
            <header class="panel-head">
                <h2 class="panel-title">{id.clone()}</h2>
                <button
                    class="peek-btn panel-refresh"
                    title="Refresh"
                    on:click=move |_| set_reload.update(|n| *n += 1)
                >
                    "↻"
                </button>
            </header>

            <Suspense fallback=|| view! { <p class="muted panel-note">"Loading…"</p> }>
                {
                    let api = api.clone();
                    move || {
                        let api = api.clone();
                        Suspend::new(async move {
                            match data.await {
                                Ok(storyline) => {
                                    view! { <StorylineForm api storyline set_session on_saved /> }
                                        .into_any()
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
pub fn Templates(jwt: JWT, set_session: WriteSignal<Option<JWT>>) -> impl IntoView {
    let api = TemplateApi::new(jwt, URL(ENV.api_base.to_string()));

    let (ids, set_ids) = signal::<Vec<String>>(Vec::new());
    let (list_status, set_list_status) = signal(String::new());
    let (selected, set_selected) = signal::<Option<TemplateTarget>>(None);

    let selected_existing_id = Memo::new(move |_| match selected.get() {
        Some(TemplateTarget::Existing(id)) => Some(id),
        _ => None,
    });

    // fetch the id list once, on mount
    {
        let api = api.clone();
        Effect::new(move |_| {
            let api = api.clone();
            spawn_local(async move {
                match api.list().await {
                    Ok(list) => set_ids.set(list),
                    Err(DomainError::NotAllowed(_)) => {
                        session::clear();
                        set_session.set(None);
                    }
                    Err(e) => set_list_status.set(format!("Failed to load templates: {e}")),
                }
            });
        });
    }

    let on_saved = Callback::new(move |id: String| {
        set_ids.update(|list| {
            if !list.iter().any(|x| x == &id) {
                list.push(id.clone());
            }
        });
        set_selected.set(Some(TemplateTarget::Existing(id)));
    });

    let on_create = Callback::new(move |id: String| {
        set_selected.set(Some(TemplateTarget::New(blank_storyline(id))));
    });

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
                <nav class="dash-nav">
                    <a class="ghost-btn" href="/hq/controls">
                        "Controls"
                    </a>
                    <a class="ghost-btn" href="/hq/templates">
                        "Templates"
                    </a>
                </nav>
                <button class="ghost-btn" on:click=logout>
                    "Log out"
                </button>
            </header>

            <section class="dash-body wide">
                <h1>"Templates"</h1>

                {move || {
                    let s = list_status.get();
                    (!s.is_empty()).then(|| view! { <p class="auth-error">{s}</p> })
                }}

                <div class="tpl-picker">
                    <TemplateSelect ids selected=selected_existing_id set_selected />
                    <NewTemplateForm ids on_create />
                </div>

                {move || {
                    selected
                        .get()
                        .map(|target| match target {
                            TemplateTarget::Existing(id) => {
                                view! {
                                    <ExistingTemplateEditor api=api.clone() id set_session on_saved />
                                }
                                    .into_any()
                            }
                            TemplateTarget::New(storyline) => {
                                let label = storyline.id.0.clone();
                                view! {
                                    <section class="panel template-panel">
                                        <header class="panel-head">
                                            <h2 class="panel-title">{label}</h2>
                                            <span class="tag">"unsaved"</span>
                                        </header>
                                        <StorylineForm api=api.clone() storyline set_session on_saved />
                                    </section>
                                }
                                    .into_any()
                            }
                        })
                }}
            </section>
        </main>
    }
}
