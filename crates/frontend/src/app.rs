use leptos::prelude::*;
use leptos_router::{
    components::{Redirect, Route, Router, Routes},
    path,
};
use pkg::auth::claims::JWT;

use crate::{
    pages::{controls::Controls, login::Login, templates::Templates},
    session,
};

/// Root of the app. Holds the session as a signal: no token (or a token the API later
/// rejects) shows `<Login>`, a token mounts the router. Signing in or out flips the
/// signal, which swaps the whole shell.
#[component]
pub fn App() -> impl IntoView {
    let (auth, set_auth) = signal(session::load());

    view! {
        {move || match auth.get() {
            Some(jwt) => view! { <Routed jwt=jwt set_session=set_auth /> }.into_any(),
            None => view! { <Login set_session=set_auth /> }.into_any(),
        }}
    }
}

/// Authenticated shell. Pages: `/hq/controls` and `/hq/templates`; `/` redirects
/// to controls.
#[component]
fn Routed(jwt: JWT, set_session: WriteSignal<Option<JWT>>) -> impl IntoView {
    view! {
        <Router>
            <Routes fallback=|| view! { <p class="muted">"Not found"</p> }>
                <Route
                    path=path!("/hq/controls")
                    view={
                        let jwt = jwt.clone();
                        move || view! { <Controls jwt=jwt.clone() set_session /> }
                    }
                />
                <Route
                    path=path!("/hq/templates")
                    view=move || view! { <Templates jwt=jwt.clone() set_session /> }
                />
                <Route path=path!("/") view=|| view! { <Redirect path="/hq/controls" /> } />
            </Routes>
        </Router>
    }
}
