//! Bridge to an injected Ethereum wallet, using **EIP-6963** multi-wallet discovery.
//!
//! Reading `window.ethereum` directly breaks when several wallet extensions are
//! installed - they race to own that property and the losers throw
//! (`evmAsk.js … selectExtension`). Instead we dispatch `eip6963:requestProvider`,
//! collect every `eip6963:announceProvider` the extensions answer with, and call
//! `.request()` on a specific provider (MetaMask if present, otherwise the first
//! announced, otherwise legacy `window.ethereum`).
//!
//! Only two RPCs are needed for login: `eth_requestAccounts` and `personal_sign`.

use js_sys::{Array, Reflect};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = r#"
// { info: {uuid,name,rdns,icon}, provider } for each announced wallet
let _providers = [];
// the provider chosen at connect(), reused for later requests
let _active = null;
let _armed = false;

function arm() {
  if (_armed || typeof window === 'undefined') return;
  _armed = true;
  window.addEventListener('eip6963:announceProvider', (e) => {
    const d = e.detail;
    if (!d || !d.info || !d.provider) return;
    if (!_providers.some((p) => p.info.uuid === d.info.uuid)) {
      _providers.push({ info: d.info, provider: d.provider });
    }
  });
  // ask now so the list is usually ready by the time the user clicks
  window.dispatchEvent(new Event('eip6963:requestProvider'));
}
arm();

function pick(preferred) {
  if (preferred) {
    const hit = _providers.find((p) => p.info.rdns === preferred);
    if (hit) return hit.provider;
  }
  const mm = _providers.find((p) => p.info.rdns === 'io.metamask');
  if (mm) return mm.provider;
  if (_providers.length) return _providers[0].provider;
  if (typeof window !== 'undefined' && window.ethereum) return window.ethereum;
  return null;
}

export function wallet_any_available() {
  arm();
  return _providers.length > 0 || (typeof window !== 'undefined' && !!window.ethereum);
}

export async function wallet_connect(preferred) {
  arm();
  // re-ask and give late/just-enabled extensions a moment to answer
  window.dispatchEvent(new Event('eip6963:requestProvider'));
  await new Promise((r) => setTimeout(r, 300));

  _active = pick(preferred || null);
  if (!_active) throw new Error('No Ethereum wallet extension detected');
  return await _active.request({ method: 'eth_requestAccounts', params: [] });
}

export async function wallet_request(method, params) {
  if (!_active) throw new Error('wallet is not connected');
  return await _active.request({ method, params: params ?? [] });
}
"#)]
extern "C" {
    fn wallet_any_available() -> bool;

    #[wasm_bindgen(catch)]
    async fn wallet_connect(preferred: Option<String>) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn wallet_request(method: &str, params: JsValue) -> Result<JsValue, JsValue>;
}

/// Is at least one injected wallet reachable (announced via EIP-6963, or a legacy
/// `window.ethereum`)?
pub fn available() -> bool {
    wallet_any_available()
}

/// Discover wallets, pick one (MetaMask preferred), prompt it to connect, and return
/// its primary account `0x`-prefixed exactly as the wallet reports it. The chosen
/// provider is remembered for [`personal_sign`].
pub async fn connect() -> Result<String, String> {
    let accounts = wallet_connect(None).await.map_err(describe)?;
    Array::from(&accounts)
        .get(0)
        .as_string()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "wallet returned no account".to_string())
}

/// `personal_sign` `message` with `address` on the provider chosen by [`connect`].
/// Returns the 65-byte `r || s || v` signature as `0x…` hex.
pub async fn personal_sign(message: &str, address: &str) -> Result<String, String> {
    let msg_hex = format!("0x{}", hex::encode(message.as_bytes()));
    let params = Array::of2(&JsValue::from_str(&msg_hex), &JsValue::from_str(address));
    let sig = wallet_request("personal_sign", params.into())
        .await
        .map_err(describe)?;
    sig.as_string()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "wallet returned no signature".to_string())
}

/// Best-effort human message out of a rejected provider promise (usually
/// `{ code, message }`); falls back to a generic string.
fn describe(err: JsValue) -> String {
    Reflect::get(&err, &JsValue::from_str("message"))
        .ok()
        .and_then(|m| m.as_string())
        .or_else(|| err.as_string())
        .unwrap_or_else(|| "wallet request was rejected".to_string())
}
