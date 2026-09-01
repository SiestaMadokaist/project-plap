//! Thin bridge to an injected EIP-1193 provider (`window.ethereum`).
//!
//! Only two calls are needed for login: `eth_requestAccounts` to learn the wallet
//! address, and `personal_sign` to sign the server's challenge. The wallet applies the
//! EIP-191 frame itself, so we hand it the raw [`ServerChallenge::sign_message`] string
//! (hex-encoded so it is treated as bytes, not re-interpreted as text).

use js_sys::{Array, Reflect};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = r#"
export function wallet_available() {
  return typeof window !== 'undefined' && typeof window.ethereum !== 'undefined';
}
export async function wallet_request(method, params) {
  if (typeof window === 'undefined' || !window.ethereum) {
    throw new Error('No injected Ethereum wallet found');
  }
  return await window.ethereum.request({ method, params: params ?? [] });
}
"#)]
extern "C" {
    fn wallet_available() -> bool;

    #[wasm_bindgen(catch)]
    async fn wallet_request(method: &str, params: JsValue) -> Result<JsValue, JsValue>;
}

/// Is there an injected `window.ethereum` provider in this page?
pub fn available() -> bool {
    wallet_available()
}

/// Prompt the wallet to connect and return its primary account, `0x`-prefixed exactly
/// as the wallet reports it (mixed-case EIP-55 checksum).
pub async fn connect() -> Result<String, String> {
    let raw = wallet_request("eth_requestAccounts", JsValue::NULL)
        .await
        .map_err(describe)?;
    Array::from(&raw)
        .get(0)
        .as_string()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "wallet returned no account".to_string())
}

/// `personal_sign` `message` with `address`. Returns the 65-byte `r || s || v`
/// signature as `0x…` hex.
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
