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

make run-api      # local Lambda server for api on http://localhost:9000 (cargo lambda watch)
make invoke-api   # invoke api locally with /tmp/api.json
make invoke-ws    # invoke ws locally with events/ws.json
make invoke-cron  # invoke cron locally with events/cron.json

make clean        # remove build artifacts and dist/
```

`make plan` and `make deploy` always build first — running them standalone is safe.

## The `datatransfer` feature

Model/output file transfer (upload, download, `abs_path`) is gated behind a Cargo feature flag, `datatransfer`, and is required by (and only by) the `diffusion-agent` binary — `Cargo.toml` sets `required-features = ["datatransfer"]` on it. The 3 Lambda binaries never compile that code path at all.

Why it's split out:
- **Cost.** Data transfer between EC2 and S3 is where this project wants that traffic to happen, not through Lambda — hence `diffusion-agent`, not `api`/`cron`, does the heavy model/output transfers.
- **Speed.** For large files (model weights, generated images), `S3Storage::upload`/`download` shell out to `aws s3 cp`, which parallelizes over multiple connections and is significantly faster than the single-connection S3 SDK client. For small payloads (JSON metadata, translated text), `S3Storage::read`/`write` use the SDK directly instead — spawning a subprocess isn't worth it at that size.

`make lint`/`make test`/`make coverage` all pass `--features datatransfer` so that code path stays covered; `make check`/`make build` (for the Lambda binaries) intentionally don't.

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
