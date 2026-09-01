use leptos::prelude::*;

use crate::{
    pages::{dashboard::Dashboard, login::Login},
    session,
};

/// Root of the app. Holds the session as a signal: no token (or a token the API later
/// rejects) shows `<Login>`, a token shows `<Dashboard>`. Signing in or out just flips
/// this signal, so there is nothing to "navigate" to.
#[component]
pub fn App() -> impl IntoView {
    let (auth, set_auth) = signal(session::load());

    view! {
        {move || match auth.get() {
            Some(jwt) => view! { <Dashboard jwt=jwt set_session=set_auth /> }.into_any(),
            None => view! { <Login set_session=set_auth /> }.into_any(),
        }}
    }
}
