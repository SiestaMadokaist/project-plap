//! Data Transfer Objects shared between `backend` and `frontend`.
//!
//! Types here define the wire format for the HTTP/WS API. They depend on
//! `domain` for core types but stay free of any runtime (AWS SDK, leptos, etc.)
//! so both the server and the wasm client can compile against them.
pub mod httpcode;
pub mod resources;
pub mod response;
