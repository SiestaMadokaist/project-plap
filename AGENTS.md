# Agent instructions — rust.lambda

## Every `UsecaseAPI` payload/response must be a dedicated DTO

`UsecaseAPI<C>` (`crates/backend/src/application/ports/usecase.rs`) is the
contract between a usecase and the route handler that calls it
(`crates/backend/src/bin/api/routes/authorized.rs`). Both the request payload
a usecase takes and the `C` it returns **must** be a type defined in the `dto`
crate (`crates/dto/src/resources/...`) — never a bare `domain::` type, even
when the DTO would just be a one-field wrapper around it.

```rust
// dto/src/resources/computes.rs
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComputeDTO(pub ComputeInstance); // wraps domain::commands::compute::ComputeInstance
impl DTO for ComputeDTO {}
```

is correct even though `ComputeDTO` adds nothing but a name. Implementing
`UsecaseAPI<ComputeInstance>` directly against the domain type is not
acceptable, no matter how tempting the shortcut looks for a "simple" endpoint.

Payload types follow the same rule and use `json_type!` (from `pkg::macros`)
to get `TryFrom<serde_json::Value>` for the `payload.try_into()?` call in the
route handler:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct DeletePayload {
    pub action_id: ActionId, // domain::commands::command::ActionId
}
json_type!(DeletePayload);
```

Existing examples to follow: `dto::resources::commands::{DeletePayload,
GetListResponse, CpModelPayload, CpModelResponse}`, `dto::resources::models::
{GetListPayload, PreviewPayload, PreviewResponse}`, `dto::resources::computes::
{ComputeDTO, ComputeControlPayload, ComputeListPayload, ComputeListResponse}`.

### Why

`crates/frontend/src/api/plap.rs` (`PlapApi`) pins every call's expected
response type to a `dto::resources::*` type at the call site, e.g.:

```rust
self.send::<models::GetListResponse>(builder).await?.get()
```

If a usecase's `exec()` returned a `domain::` type directly and a later
refactor changed *which* domain type backs that endpoint (or changed the
domain type's own shape for an unrelated reason), every frontend call site
that hardcodes the old type would silently expect the wrong shape — nothing
would catch it at compile time, because the domain type isn't the contract,
it's an implementation detail of the usecase.

Routing every usecase through its own DTO means the wire contract for an
endpoint is a single, explicit type in `dto`, decoupled from whatever domain
type currently implements it. Endpoint X always returns `DtoX`; what `DtoX`
wraps internally can change freely without breaking `PlapApi`'s assumptions
about what endpoint X returns.

