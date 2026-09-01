//! Where the session JWT lives on the client: `localStorage`.
//!
//! `localStorage` is readable by any script on the origin, so a stored JWT is exposed
//! to XSS. That is the accepted trade-off here (no SSR, no cookie backend); keep the
//! token short-lived server-side and treat any `NotAllowed` from the API as "re-login".

use gloo_storage::{LocalStorage, Storage};
use pkg::auth::claims::JWT;

const JWT_KEY: &str = "plap.jwt";

/// The stored session token, if there is a non-empty one.
pub fn load() -> Option<JWT> {
    LocalStorage::get::<String>(JWT_KEY)
        .ok()
        .filter(|s| !s.is_empty())
        .map(JWT)
}

pub fn save(jwt: &JWT) {
    let _ = LocalStorage::set(JWT_KEY, &jwt.0);
}

pub fn clear() {
    LocalStorage::delete(JWT_KEY);
}
