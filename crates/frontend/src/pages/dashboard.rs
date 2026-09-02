use std::rc::Rc;

use domain::{
    errors::DomainError,
    storage::{DirTree, StoragePrefix},
};
use leptos::prelude::*;
use pkg::{auth::claims::JWT, types::strings::URL};

use crate::{api::plap::PlapApi, session, API_BASE};

/// Top of the browsable tree. S3 keys/prefixes come back rooted here
/// (`MODEL_PREFIX` is empty), so every path the UI shows starts with this.
const ROOT_PREFIX: &str = "";

/// `comfyui/models/` -> [("comfyui", "comfyui/"), ("models", "comfyui/models/")]
fn crumbs(prefix: &str) -> Vec<(String, String)> {
    let mut acc = String::new();
    prefix
        .trim_end_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .map(|seg| {
            acc.push_str(seg);
            acc.push('/');
            (seg.to_string(), acc.clone())
        })
        .collect()
}

/// Name of a child relative to the folder currently open.
/// `leaf("comfyui/models/loras/", "comfyui/models/")` -> `"loras/"`.
fn leaf<'a>(full: &'a str, current: &str) -> &'a str {
    full.strip_prefix(current).unwrap_or(full)
}

#[component]
fn Crumbs(
    prefix: ReadSignal<StoragePrefix>,
    set_prefix: WriteSignal<StoragePrefix>,
) -> impl IntoView {
    move || {
        let parts = crumbs(&prefix.get().0);
        let last = parts.len().saturating_sub(1);
        view! {
            <nav class="crumbs">
                <span class="crumbs-label">"path"</span>
                {parts
                    .into_iter()
                    .enumerate()
                    .map(|(i, (name, path))| {
                        let go = move |_| set_prefix.set(StoragePrefix(path.clone()));
                        view! {
                            <span class="crumb-wrap">
                                {(i > 0).then(|| view! { <span class="crumb-sep">"/"</span> })}
                                <button class="crumb" class:current=i == last on:click=go>
                                    {name}
                                </button>
                            </span>
                        }
                    })
                    .collect_view()}
            </nav>
        }
    }
}

#[component]
fn FileTree(
    tree: DirTree,
    current: String,
    set_prefix: WriteSignal<StoragePrefix>,
) -> impl IntoView {
    let DirTree { paths, prefixes } = tree;
    let has_folders = !prefixes.is_empty();
    let has_files = !paths.is_empty();

    let folders = prefixes
        .into_iter()
        .map(|p| {
            let full = p.0;
            let name = leaf(&full, &current).to_string();
            let go = move |_| set_prefix.set(StoragePrefix(full.clone()));
            view! {
                <li class="row folder" on:click=go>
                    <span class="row-ico">"📁"</span>
                    <span class="row-name">{name}</span>
                    <span class="row-hint">"open"</span>
                </li>
            }
        })
        .collect_view();

    let files = paths
        .into_iter()
        .map(|p| {
            let name = leaf(&p.0, &current).to_string();
            view! {
                <li class="row file">
                    <span class="row-ico">"📄"</span>
                    <span class="row-name">{name}</span>
                </li>
            }
        })
        .collect_view();

    view! {
        <div class="tree">
            {(!has_folders && !has_files)
                .then(|| view! { <p class="muted empty-note">"This folder is empty."</p> })}
            {has_folders
                .then(|| {
                    view! {
                        <div class="tree-group">
                            <p class="tree-label">"Folders"</p>
                            <ul class="row-list">{folders}</ul>
                        </div>
                    }
                })}
            {has_files
                .then(|| {
                    view! {
                        <div class="tree-group">
                            <p class="tree-label">"Files"</p>
                            <ul class="row-list">{files}</ul>
                        </div>
                    }
                })}
        </div>
    }
}

#[component]
pub fn Dashboard(jwt: JWT, set_session: WriteSignal<Option<JWT>>) -> impl IntoView {
    let api = Rc::new(PlapApi::new(jwt, URL(API_BASE.to_string())));
    let (prefix, set_prefix) = signal(StoragePrefix(ROOT_PREFIX.into()));

    let models = LocalResource::new({
        let api = api.clone();
        move || {
            let api = api.clone();
            let value = prefix.get();
            async move { api.list_models(value).await }
        }
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
                <button class="ghost-btn" on:click=logout>
                    "Log out"
                </button>
            </header>

            <section class="dash-body">
                <h1>"Models"</h1>

                <div class="browser">
                    <Crumbs prefix set_prefix />

                    <Suspense fallback=|| {
                        view! { <p class="muted browse-note">"Loading…"</p> }
                    }>
                        {move || Suspend::new(async move {
                            match models.await {
                                Ok(resp) => {
                                    let current = prefix.get_untracked().0;
                                    view! { <FileTree tree=resp.0 current set_prefix /> }.into_any()
                                }
                                Err(DomainError::NotAllowed(_)) => {
                                    request_animation_frame(move || {
                                        session::clear();
                                        set_session.set(None);
                                    });
                                    view! {
                                        <p class="muted browse-note">
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
                        })}
                    </Suspense>
                </div>
            </section>
        </main>
    }
}
