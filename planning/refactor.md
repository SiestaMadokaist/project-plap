# Refactor plan: domain/dto separation + typed routes

Two goals, one order of work:

1. Keep `domain` pure and light, so `frontend -> dto -> domain` compiles to wasm without crypto or vendor code.
2. Lock each route to its payload and response types, shared by frontend and backend, so drift is a compile error.

## Decisions already made

- DTOs stay thin wrappers: `ComputeDTO(pub ComputeInstance)`. A domain change that alters the wire shape is acceptable because the frontend and backend share the type and the compiler catches drift.
- PII is handled by discipline: a sensitive value gets its own wire-safe struct. Optional hardening: sensitive newtypes get a redacted `Debug` and no `Serialize` derive unless they are meant to cross the wire.
- Dependency direction is `dto -> domain`. `domain` never imports `dto`.
- No logging or other side effects in `domain` or `dto`. Functions return values (a `Result` with an error enum) and `backend` logs.

## Layer rules

| | `domain` | `dto` | `backend` |
|---|---|---|---|
| Holds | types, invariants, pure rules, ports (traits) | payload/response structs, route markers, `From`/`TryFrom` | usecases, I/O, signing, logging, domain<->dto mapping at the route edge |
| I/O, clock, randomness, secrets | no (take `now` as a parameter) | no | yes |
| Vendor code (aws, reqwest, civitai, comfy) | no | no | yes |
| Must compile for wasm | yes | yes | no |

Test: could the frontend call this? If it needs a secret, the network or the clock, it does not belong in `domain` or `dto`.

Check after each phase: `cargo tree -p frontend` must not show `k256`, `jsonwebtoken`, `aws-*` or `reqwest`.

## Phase 1: workspace hygiene (mechanical, low risk)

- Add `[workspace.dependencies]` for `serde`, `serde_json`, `chrono`, `thiserror`, `tracing`, `hex`, `anyhow` and use `{ workspace = true }` in every crate.
- Add `[workspace.lints]` (move the `warnings = deny` policy from `backend`) and opt every crate in.
- Find and remove leftovers: `sqlx`/`arkhamstep`, the duplicate `future` feature, `notify`, unused `jsonwebtoken`/`k256`/`sha3` duplicates between `pkg` and `backend`.

## Phase 2: slim `pkg` so the wasm build stops pulling crypto

- Split `pkg` by reason to change:
  - leaf types crate (keep the name `pkg` or rename to `types`): `Timestamp`, `Second`, `MilliSecond`, ids, `URL`, `json_type!`/`id_type!`/`displayable!` macros. No crypto, wasm-safe.
  - auth/crypto (`auth/ecdsa`, `jsonwebtoken`, `k256`, `sha3`, `getrandom`, `uuid`): `backend` only. The wallet signs in JS, so the frontend only handles signature strings.
  - integrations (`civitai`, `comfy`, `exif`): move into `backend` (or an `integrations` crate if the diffusion agent needs them separately).
- `domain` stops importing `pkg::civitai` and `pkg::auth`. If `domain` needs `AuthClaims` or `Username`, move those small value types into the leaf crate.
- Remove from `frontend/Cargo.toml` the `getrandom` 0.2/0.3/0.4 entries and the `.cargo/config.toml` cfg hack once `cargo tree -p frontend` is clean.

## Phase 3: move logic out of `dto` into `domain`, and secrets into `backend`

Start with `dto/resources/login.rs` (`ServerChallenge`, `ChallengeAnswer`):

- `domain/login.rs`: the structs and the pure rules.
  - `canonical_bytes()` (what gets signed)
  - `check(now, tolerance, max_ttl) -> Result<(), ChallengeError>` (clock drift, expiry, TTL), replacing the `bool`-returning `validate` and its `tracing` call.
- `backend` usecase: `mint` (signs with the server key) and `verify_signature`. Add a `ChallengeSigner` port in `domain` only if a test needs a fake signer.
- `dto/login.rs`: `pub struct ServerChallengeDTO(pub ServerChallenge)` and the payload types. No methods beyond `From`/`TryFrom`.
- Repeat for other DTOs that hold rules, signing or logging. Audit `dto/src` for `tracing`, `Utc::now`, key handling.

Also: `frontend` should depend on `dto` and the leaf types crate, not on `domain` directly. Re-export from `dto` what the frontend needs (for example `ComputeRegion`).

## Phase 4: typed routes (`Endpoint` trait)

In `dto`:

```rust
// dto/src/endpoint.rs
pub enum Method { Get, Post, Put, Delete }

pub trait Endpoint {
    type Payload: Serialize + DeserializeOwned;
    type Response: Serialize + DeserializeOwned;
    const METHOD: Method;
    const PATH: &'static str;
}
```

- One marker type per route, next to its DTOs (`dto/resources/<resource>.rs`), not in a central routes file:

```rust
pub struct ComputeList;
impl Endpoint for ComputeList {
    type Payload = ComputeListPayload;
    type Response = ComputeListResponse;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/computes/list";
}
```

- Routes with no body use `type Payload = ()`. Routes with path params put the id in `Payload`, or make `PATH` a function of the payload.
- Optional later: `const AUTH: Auth` if public/authorized routing starts duplicating decisions.

Frontend (`PlapApi`): replace `send::<Response>(builder)` with one generic call:

```rust
async fn call<E: Endpoint>(&self, payload: &E::Payload) -> Result<E::Response, ApiError>
```

so the URL, method, payload type and response type all come from `E`.

Backend: a `RouterExt::endpoint::<E, _, _>(handler)` helper in `bin/api/routes/` that registers `E::PATH` with `E::METHOD` and takes a handler of type `Fn(E::Payload) -> Result<E::Response, ApiError>`:

```rust
Router::new()
    .endpoint::<ComputeList, _, _>(compute_list)
    .endpoint::<ComputeControl, _, _>(compute_control)
```

Open points to check against the real code before building this:
- Authorized routes need the auth claims. Either keep the auth middleware as a `.layer(...)` and pass claims via a request extension, or widen the helper to `handler(claims, payload)`. Read `routes/authorized.rs` first.
- Reconcile with `UsecaseAPI<C>`: its payload and return type should be `E::Payload` / `E::Response`. Keep the AGENTS.md rule (dedicated DTOs, no bare `domain::` types).
- GET routes without a body need a separate helper (`.endpoint_get`) or query-string reading.
- `Json<E::Payload>` replaces the `payload.try_into()?` from `serde_json::Value` at the route. `json_type!` can stay for places that still need it.

Migration: convert one route end to end (marker, frontend call, backend registration) as a template, then convert the rest one by one. Both styles can coexist while migrating.

## Phase 5 (optional, only if compile time or features hurt): split `backend`

- `application` (usecases and ports, no AWS), adapter crates (`infra-aws`, `infra-ec2`), thin binary crates (`api`, `ws`, `cron`, `agent`).
- This removes the need for the `datatransfer` and `remote-ec2` feature flags because each binary depends only on what it uses.
- Large change; the current feature flags work, so do this last or not at all.
- Delete the Lambda path (`lambda_runtime`, `ws`, `cron` packaging) once the move to axum is confirmed.

## Order

1. Phase 1 (workspace hygiene)
2. Phase 2 (slim `pkg`), verify with `cargo tree -p frontend`
3. Phase 3 (logic out of `dto`), starting with login
4. Phase 4 (typed routes), one route first
5. Phase 5 only if needed

Update AGENTS.md at the end of Phase 4 to add the `Endpoint` marker rule next to the existing DTO rule, so new routes follow it.
