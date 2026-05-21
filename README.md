# rust-api

AWS Lambda functions written in Rust, deployed via Terraform.

## Functions

| Function | Trigger | Timeout |
|---|---|---|
| `api` | HTTP API Gateway v2 (catch-all) | 10s |
| `ws` | WebSocket API Gateway v2 | 10s |
| `cron` | EventBridge schedules | 900s |

Runtime: `provided.al2023`, architecture: `arm64` (Graviton), memory: `512MB`.

## Prerequisites

- [Rust](https://rustup.rs/)
- [cargo-lambda](https://www.cargo-lambda.info/): `cargo install cargo-lambda`
- [Terraform](https://developer.hashicorp.com/terraform/install) >= 1.5
- AWS credentials configured (`~/.aws/credentials` or env vars)

## First-time setup

```bash
make tf-init   # initialises Terraform, pulls providers, sets up S3 backend
```

Terraform state is stored in `s3://serverless.ramadoka.com/terraform/rust-api/terraform.tfstate`.

## Workflow

```bash
make verify        # cargo check + clippy
make fmt           # cargo fmt
make test          # cargo test
make build         # cross-compile to arm64 Linux (target/lambda/<fn>/bootstrap)
make run           # local Lambda server on http://localhost:9000
make invoke-api    # invoke api locally with events/api.json
make invoke-ws     # invoke ws locally with events/ws.json
make invoke-cron   # invoke cron locally with events/cron.json
make plan          # build + package + terraform plan
make deploy        # build + package + terraform apply
make clean         # remove build artifacts and dist/
```

`make plan` and `make deploy` always build first — running them standalone is safe.

## Infrastructure

```
terraform/
  main.tf              provider (AWS ap-southeast-1), S3 backend
  variables.tf         stage, region, memory, etc.
  iam.tf               Lambda execution role + inline policy
  lambda.tf            3 Lambda functions
  api_gateway_http.tf  HTTP API v2 → api Lambda
  api_gateway_ws.tf    WebSocket API v2 → ws Lambda
  cloudwatch.tf        Log groups (7-day retention)
  eventbridge.tf       4 cron rules → cron Lambda
  outputs.tf           api_endpoint, ws_endpoint
```

### IAM permissions

Mirrors `serverless.yml`:

- `ce:GetCostAndUsage`
- `dynamodb:*` on `production-*` tables
- `ec2:*`
- `iam:PassRole`
- `s3:*` on project buckets
- `execute-api:ManageConnections` on the WebSocket API (for sending messages back to clients)

### Cron schedules

| Rule | Schedule (UTC) | Input |
|---|---|---|
| ocr_service_morning | 02:00 daily | `cron/ocr-service` |
| ocr_service_afternoon | 14:00 daily | `cron/ocr-service` |
| mldn | 01:00 daily | `cron/mldn` |
| autosync | 13:00 daily | `cron/autosync` |

## Endpoints

After `make deploy`, Terraform outputs:

```
api_endpoint = "https://<id>.execute-api.ap-southeast-1.amazonaws.com"
ws_endpoint  = "wss://<id>.execute-api.ap-southeast-1.amazonaws.com/prod"
```

> Custom domain / Route 53 is not configured — endpoints use the default API Gateway URLs.

## Environment

Each Lambda receives `ENV_PATH=".env.prod"` and `RUST_LOG="info"`. Place secrets in `.env.prod` (gitignored).

To deploy a different stage:

```bash
make deploy STAGE=staging
```
