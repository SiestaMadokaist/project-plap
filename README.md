# rust-api

3 AWS Lambda functions plus a standalone image-generation agent running on an EC2 spot instance, all written in Rust and deployed via Terraform.

## Components

| Binary | Runs on | Trigger | Timeout |
|---|---|---|---|
| `api` | Lambda | HTTP API Gateway v2 (catch-all) | 10s |
| `ws` | Lambda | WebSocket API Gateway v2 | 10s |
| `cron` | Lambda | EventBridge schedules | 900s |
| `diffusion-agent` | EC2 (persistent spot instance) | long-running process, not event-triggered | n/a |

Lambda runtime: `provided.al2023`, architecture: `arm64` (Graviton), memory: `512MB`.

`diffusion-agent` polls a DynamoDB command queue (`agent_commands` table) for image-generation work, drives a local Stable Diffusion webui (A1111/ComfyUI), and pushes models/outputs to/from S3. It's not deployed with `cargo-lambda` since it isn't a Lambda — see [Diffusion agent](#diffusion-agent) below.

## Prerequisites

- [Rust](https://rustup.rs/)
- [cargo-lambda](https://www.cargo-lambda.info/): `cargo install cargo-lambda` (for the 3 Lambda binaries only)
- [Terraform](https://developer.hashicorp.com/terraform/install) >= 1.5
- AWS credentials configured (`~/.aws/credentials` or env vars)

## First-time setup

```bash
make tf-init   # initialises Terraform, pulls providers, sets up S3 backend
```

Terraform state is stored in `s3://serverless.ramadoka.com/terraform/rust-api/terraform.tfstate`.

## Workflow

```bash
make verify       # check + lint + test + typos
make check        # cargo check
make lint         # cargo clippy --features datatransfer -D warnings (a few style lints allowed, see Makefile)
make fix          # cargo fix --features datatransfer
make fmt          # cargo fmt
make test         # cargo test --features datatransfer
make coverage     # cargo llvm-cov --features datatransfer --html

make build        # cross-compile api/ws/cron to arm64 Linux (target/lambda/<fn>/bootstrap)
make package      # build + zip into dist/
make plan         # package + terraform plan
make deploy       # package + terraform apply
make deploy-bin   # build diffusion-agent, ship binary + .env.diffusion to S3 (see below)

make build-api    # build/package/deploy just api, skipping ws/cron (still WIP path)
make package-api
make deploy-api

make run-api      # axum server for api on http://localhost:3000 (BIND_ADDR to change)
make invoke-api   # invoke api locally with /tmp/api.json
make invoke-ws    # invoke ws locally with events/ws.json
make invoke-cron  # invoke cron locally with events/cron.json

make clean        # remove build artifacts and dist/
```

`make plan` and `make deploy` always build first — running them standalone is safe.

## The `datatransfer` feature

Model/output file transfer (upload, download, `abs_path`) is gated behind a Cargo feature flag, `datatransfer`, and is required by (and only by) the `diffusion-agent` binary — `Cargo.toml` sets `required-features = ["datatransfer", "remote-ec2"]` on it (see the next section for the second feature). The 3 Lambda binaries never compile that code path at all.

Why it's split out:
- **Cost.** Data transfer between EC2 and S3 is where this project wants that traffic to happen, not through Lambda — hence `diffusion-agent`, not `api`/`cron`, does the heavy model/output transfers.
- **Speed.** For large files (model weights, generated images), `S3Storage::upload`/`download` shell out to `aws s3 cp`, which parallelizes over multiple connections and is significantly faster than the single-connection S3 SDK client. For small payloads (JSON metadata, translated text), `S3Storage::read`/`write` use the SDK directly instead — spawning a subprocess isn't worth it at that size.

`make lint`/`make test`/`make coverage` all pass `--features datatransfer` so that code path stays covered; `make check`/`make build` (for the Lambda binaries) intentionally don't.

## The `remote-ec2` feature

Everything that talks to EC2 is gated behind a second Cargo feature, `remote-ec2`, which is off by default:

- `infras::compute::ec2` (`EC2MultiRegion`, used by the API to launch / control / list instances)
- `infras::compute_agent::ec2` (`EC2Agent`, used by `diffusion-agent` to read its own tags on the instance)
- the `aws-sdk-ec2` dependency itself (`optional = true`, enabled via `remote-ec2 = ["dep:aws-sdk-ec2"]`)

Why it's split out: **`aws-sdk-ec2` is the biggest compile-time cost in the workspace.** The crate is huge, and building it dominates a cold build and is a large part of an incremental check after the cache is cleared. Hiding it behind a feature means a plain `cargo check` / `cargo build` / `cargo run` doesn't compile it at all, so the normal edit-compile loop stays fast. You only pay for it when you build with `--features remote-ec2`.

What happens without the feature:
- `infras::compute::DefaultEngines` resolves to `NoEngines` (`infras/compute/none.rs`) instead of `EC2MultiRegion`. It never returns an engine, so the compute usecases (launch, control, list) behave as if the region isn't configured. Everything else in the API works as normal.
- `aws-sdk-ec2` isn't in the dependency graph (`cargo tree -i aws-sdk-ec2` finds nothing).

Where it is turned on:
- `diffusion-agent` has `required-features = ["datatransfer", "remote-ec2"]`, so it is skipped unless both are on. `make deploy-bin` builds it with `--features datatransfer,remote-ec2`.
- Only the deploy path builds with `--features remote-ec2`: `make plan`, `make deploy` and `make deploy-api` set it through the Makefile's `FEATURES` variable, so the deployed `api` can really launch, control and list instances (without it, it would silently use `NoEngines`). Plain `make build` / `make build-api` / `make package` leave it off to keep local builds fast; pass it yourself with `make build FEATURES='--features remote-ec2'`.
- CI clippy and test run with `--features datatransfer,remote-ec2`, so the EC2 code is always checked before merge.
- The local `make lint`, `make test`, `make coverage` and `make fix` only pass `datatransfer`. They skip the EC2 code on purpose to stay fast. To check it locally: `cargo clippy -p backend --features datatransfer,remote-ec2`.

## Infrastructure

```
terraform/
  main.tf              provider (AWS ap-southeast-1), S3 backend
  variables.tf         stage, region, memory, etc.
  lambda.tf            3 Lambda functions
  api_gateway_http.tf  HTTP API v2 → api Lambda
  api_gateway_ws.tf    WebSocket API v2 → ws Lambda
  cloudwatch.tf        Log groups (7-day retention)
  eventbridge.tf       cron rules → cron Lambda
  outputs.tf           api_endpoint, ws_endpoint, lambda ARNs
```

### Cron schedules

| Rule | Schedule (UTC) | Input |
|---|---|---|
| translate_deathflag | `cron(30 * * * ? *)` (hourly, :30) | `cron/translate`, novel `n4449cj` |
| translate_re0 | `cron(0 * * * ? *)` (hourly, :00) | `cron/translate`, novel `n2267be` |

## Diffusion agent

`diffusion-agent` is a plain Rust binary (not a Lambda) meant to run on a persistent EC2 spot instance:

- Polls the `agent_commands` DynamoDB table for queued work and drives image generation against a local A1111/ComfyUI instance.
- Downloads models from / uploads outputs to S3 using the `datatransfer` feature described above.
- Deployed with `make deploy-bin`: builds the release binary, runs a `SANITY_RUN` smoke check, then pushes both the binary and `.env.diffusion` to `s3://virginia-ramadoka/bin/` for the instance to pull down — no `cargo-lambda`/Lambda packaging involved.

## Trying the API with Bruno

`bruno/plap-api/` is a [Bruno](https://www.usebruno.com/) collection with one request per endpoint (open it with the Bruno app, or the Bruno VS Code extension). It uses Bruno's YAML (OpenCollection) format, which needs Bruno 3.0 or newer.

| Folder | Requests |
|---|---|
| `public/` | `challenge`, `login`, `health` (no auth) |
| `models/` | `list`, `preview` |
| `agents/` | `cp-model`, `list`, `delete` |
| `hq/` | `launch`, `control`, `list` |
| `story/` | `list`, `create`, `read`, `delete` |

Bodies are examples built from the payload types in `crates/dto`; edit them to taste.

### Running it locally

```bash
make dynamo       # start the dynamodb-local container (+ dynamodb-admin)
make dynamo-tables  # first time only: create the development-* tables (terraform/local)
make run-api      # axum server on http://localhost:3000 (ENV_PATH=.env.api.localhost)
```

Then select the **local** environment in Bruno. It defines `baseUrl` (`http://localhost:3000`) and `token`. Authorized requests send `Authorization: Bearer {{token}}`.

### Logging in

1. Send `public/challenge`. Its scripts set `iat` and `exp` from the current time (now - 5s and now + 1h) and, after the response, store `auth_code`, `auth_server_sign` and `auth_address` as runtime variables.
2. Sign the challenge `code` with your wallet.
3. Put the signature in `public/login` as `client_sign` and send it. The `challenge` object in the body must be exactly the one the server returned (same `iat` and `exp`).
4. Copy the returned token into the `token` variable of the **local** environment.

### Safe mode vs developer mode (read this if `login` gets empty variables)

Bruno runs request scripts in one of two modes, set per collection:

| | Safe mode | Developer mode |
|---|---|---|
| Scripts run in | a sandbox | Node, with full access to the machine |
| `bru.setVar(...)` shared between requests | **no**, each request is isolated (unless you use the runner) | **yes** |

This collection needs **developer mode**, because `public/challenge` stores `auth_code`, `auth_server_sign` and `auth_address` with `bru.setVar`, and `public/login` reads them as `{{auth_code}}` and so on.

**Symptoms of being in safe mode:** `login` is sent with the literal text `{{auth_code}}` (or an empty value) in its body, and the server answers with a failure. In the script console, `bru.getVar("auth_code")` prints `undefined` in `login`, even though `challenge` just succeeded.

**To fix it:** switch the collection to **Developer Mode** in the Bruno app (the Safe Mode / Developer Mode setting for the collection; Bruno may ask you to confirm), then send `challenge` again, then `login`. Variables are in memory only, so they are lost whenever Bruno restarts or you reopen the collection. Just run `challenge` again.

**Why it isn't left on by default:** developer-mode scripts can read files and run commands, so any collection you open in that mode can run arbitrary code on your machine. It's fine for this one because the scripts are in this repo (date arithmetic and `bru.setVar` calls). Keep safe mode for collections you didn't write.

**Without developer mode:** run `challenge` and `login` together in the Bruno runner, where variables carry across a run. The wallet signature step in between can't be automated, so developer mode is the practical choice.

Notes:
- **The token:** `environments/local.yml` is tracked by git, so don't commit a real token there. Mark `token` as a secret in Bruno's environment editor, or keep the filled-in file out of git.
- **Stage:** the API's `STAGE` decides which tables it uses (`<stage>-users`, ...). The local tables are created as `development-*`, so `.env.api.localhost` needs `STAGE=development` and `AWS_ENDPOINT_URL_DYNAMODB=http://localhost:40000`.

## Endpoints

After `make deploy`, Terraform outputs:

```
api_endpoint = "https://<id>.execute-api.ap-southeast-1.amazonaws.com"
ws_endpoint  = "wss://<id>.execute-api.ap-southeast-1.amazonaws.com/prod"
```

> Custom domain / Route 53 is not configured — endpoints use the default API Gateway URLs.

## Environment

Each Lambda receives `ENV_PATH=".env.prod"` and `RUST_LOG="info"`. Place secrets in `.env.prod` (gitignored). `diffusion-agent` reads `.env.diffusion` instead (see `make deploy-bin`).

To deploy a different stage:

```bash
make deploy STAGE=staging
```
