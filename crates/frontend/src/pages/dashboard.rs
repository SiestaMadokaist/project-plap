use std::rc::Rc;

use domain::{
    errors::DomainError,
    storage::{DirTree, StoragePrefix},
};
use leptos::prelude::*;
use pkg::{auth::claims::JWT, types::strings::URL};

use crate::{api::plap::PlapApi, session, API_BASE};

#[component]
fn ModelList(tree: DirTree) -> impl IntoView {
    let paths = tree.paths;
    let prefixes = tree.prefixes;
    view! {
        <ul class="model-list">
            {prefixes
                .into_iter()
                .map(|path| view! { <li class="model">{path.0}</li> })
                .collect_view()}
        </ul>
        <ul class="model-list">
            {paths
                .into_iter()
                .map(|path| view! { <li class="model">{path.0}</li> })
                .collect_view()}
        </ul>
    }
}

#[component]
pub fn Dashboard(jwt: JWT, set_session: WriteSignal<Option<JWT>>) -> impl IntoView {
    let api = Rc::new(PlapApi::new(jwt, URL(API_BASE.to_string())));
    let (prefix, set_prefix) = signal(StoragePrefix("comfyui".into()));

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
                <span class="brand"><span class="brand-mark">"⬡"</span><span class="brand-name">"Project-Plap"</span></span>
                <button class="ghost-btn" on:click=logout>"Log out"</button>
            </header>

            <section class="dash-body">
                <h1>"Models"</h1>
                <input
                    class="text-input"
                    type="text"
                    prop:value=move || prefix.get().0
                    on:input=move |ev| set_prefix.set(StoragePrefix(event_target_value(&ev)))
                />

                <Suspense fallback=|| view! { <p class="muted">"Loading models…"</p> }>
                    {move || Suspend::new(async move {
                        match models.await {
                            Ok(resp) => view! { <ModelList tree=resp.0 /> }.into_any(),
                            Err(DomainError::NotAllowed(_)) => {
                                // session is no longer valid - drop it and let <App>
                                // fall back to <Login> on the next frame.
                                request_animation_frame(move || {
                                    session::clear();
                                    set_session.set(None);
                                });
                                view! { <p class="muted">"Session expired — returning to sign in…"</p> }
                                    .into_any()
                            }
                            Err(err) => view! {
                                <p class="auth-error">{format!("Failed to load: {err}")}</p>
                            }
                            .into_any(),
                        }
                    })}
                </Suspense>
            </section>
        </main>
    }
}
