use dto::resources::login::{ClientAnswer, ReqChallenge};
use leptos::prelude::*;
use pkg::{
    auth::{
        claims::JWT,
        ecdsa::{AddressETH, HexSign},
    },
    types::{
        strings::{Hex, URL},
        time::{Second, Timestamp},
    },
};
use wasm_bindgen_futures::spawn_local;

use crate::{api::auth::AuthApi, env::ENV, session, wallet};

/// How long a requested challenge/session may span. Must sit under the server's
/// `AUTH_SESSION_TTL`; the exact value only matters for the server's sanity checks.
const REQUEST_TTL_SECONDS: i64 = 3_600;

#[derive(Clone, Copy, PartialEq)]
enum Step {
    Idle,
    Connecting,
    Requesting,
    Signing,
    Verifying,
}

impl Step {
    fn label(self) -> &'static str {
        match self {
            Step::Idle => "Connect your wallet to begin.",
            Step::Connecting => "Connecting to your wallet…",
            Step::Requesting => "Requesting a challenge…",
            Step::Signing => "Waiting for your signature…",
            Step::Verifying => "Verifying and issuing a session…",
        }
    }
}

/// Run the full handshake: connect wallet → get challenge → sign → exchange for a JWT.
/// `step` is nudged along the way so the UI can narrate progress.
async fn authenticate(step: RwSignal<Step>) -> Result<JWT, String> {
    if !wallet::available() {
        return Err(
            "No Ethereum wallet detected. Install MetaMask (or another injected wallet) and reload."
                .to_string(),
        );
    }

    step.set(Step::Connecting);
    let account = wallet::connect().await?; // "0x…", mixed-case
    let address = AddressETH(account.trim_start_matches("0x").to_lowercase());

    step.set(Step::Requesting);
    let api = AuthApi::new(URL(ENV.api_base.to_string()));
    let now = Timestamp((js_sys::Date::now() / 1000.0) as i64);
    let req = ReqChallenge::new(address, now, Second(REQUEST_TTL_SECONDS));
    let challenge = api
        .request_challenge(&req)
        .await
        .map_err(|e| format!("challenge request failed: {e}"))?;

    step.set(Step::Signing);
    let signature = wallet::personal_sign(&challenge.sign_message(), &account).await?;
    let client_sign = HexSign::new(Hex(signature))
        .map_err(|e| format!("wallet returned a malformed signature ({e:?})"))?;

    step.set(Step::Verifying);
    let answer = ClientAnswer {
        challenge,
        client_sign,
    };
    let issued = api
        .login(&answer)
        .await
        .map_err(|e| format!("login rejected: {e}"))?;

    Ok(issued.token)
}

#[component]
pub fn Login(set_session: WriteSignal<Option<JWT>>) -> impl IntoView {
    let step = RwSignal::new(Step::Idle);
    let pending = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);

    let start = move |_| {
        if pending.get_untracked() {
            return;
        }
        pending.set(true);
        error.set(None);
        step.set(Step::Connecting);
        spawn_local(async move {
            match authenticate(step).await {
                Ok(jwt) => {
                    session::save(&jwt);
                    set_session.set(Some(jwt));
                }
                Err(message) => {
                    error.set(Some(message));
                    step.set(Step::Idle);
                }
            }
            pending.set(false);
        });
    };

    view! {
        <div class="auth-shell">
            <div class="auth-card">
                <div class="brand">
                    <span class="brand-mark">"⬡"</span>
                    <span class="brand-name">"Project-Plap"</span>
                </div>

                <h1 class="auth-title">"Sign in"</h1>
                <p class="auth-sub">
                    "Authenticate with your Ethereum wallet. You'll sign a short message — "
                    "no transaction, no gas."
                </p>

                <button
                    class="wallet-btn"
                    prop:disabled=move || pending.get()
                    on:click=start
                >
                    {move || if pending.get() {
                        view! { <span class="spinner"></span><span>"Working…"</span> }.into_any()
                    } else {
                        view! { <span class="wallet-ico">"🦊"</span><span>"Connect Wallet"</span> }.into_any()
                    }}
                </button>

                <p class="auth-step" class:muted=move || !pending.get()>
                    {move || step.get().label()}
                </p>

                {move || error.get().map(|message| view! {
                    <div class="auth-error" role="alert">
                        <strong>"Could not sign you in"</strong>
                        <span>{message}</span>
                    </div>
                })}

                <p class="auth-foot">
                    "Your wallet address must already be registered on this deployment."
                </p>
            </div>
        </div>
    }
}
