STAGE     ?= production
FUNCTIONS := api ws cron
DIST_DIR  := dist

S3_BIN_BUCKET := s3://secure-storage-ramadoka/bin

TF_VARS := -var="stage=$(STAGE)"

# Pulled from terraform outputs (empty until the stack is applied).
FRONTEND_BUCKET          = $(shell cd terraform && terraform output -raw frontend_bucket 2>/dev/null)
FRONTEND_DISTRIBUTION_ID = $(shell cd terraform && terraform output -raw frontend_distribution_id 2>/dev/null)
API_URL                  = $(shell cd terraform && terraform output -raw api_url 2>/dev/null)

.PHONY: all verify check lint fmt test typos build package plan deploy run run-api watch-api \
        invoke-api invoke-ws invoke-cron tf-init clean deploy-bin \
        build-api package-api deploy-api frontend-serve frontend-build frontend-deploy \
        coverage coverage-summary coverage-lcov dynamo dynamo-tables

all: verify build

# --- code quality ---

verify: check lint test typos

check:
	cargo check -p backend

lint:
	cargo clippy -p backend --features datatransfer

fix:
	cargo fix -p backend --allow-dirty --allow-staged --features datatransfer

fmt:
	cargo fmt

test:
	cargo test -p backend --features="datatransfer"

typos:
	typos

# frontend is wasm-only (won't build for the host target). --ignore-run-fail still
# emits the report while the pkg/domain fixture tests are red.
COV := --workspace --exclude frontend --features datatransfer --ignore-run-fail

coverage:
	cargo llvm-cov $(COV) --html

coverage-summary:
	cargo llvm-cov $(COV)

coverage-lcov:
	cargo llvm-cov $(COV) --lcov --output-path lcov.info

# --- build & package ---
# Requires: cargo-lambda (cargo install cargo-lambda)

# extra cargo flags for the lambda builds. Empty by default so local builds stay fast;
# the deploy targets below set it (target-specific variables reach their prerequisites).
FEATURES ?=
build:
	cargo lambda build -p backend --release --arm64 $(FEATURES) --bin api --bin ws --bin cron

# Each function ships its own dotenv at the zip root, loaded via ENV_PATH (see lambda.tf).
# api needs the auth/storage vars in .env.api; ws/cron run on .env.production.
package: build
	mkdir -p $(DIST_DIR)
	@for fn in $(FUNCTIONS); do \
		case $$fn in api) env=.env.api ;; *) env=.env.production ;; esac; \
		echo "unzipped size $$fn: $$(du -h target/lambda/$$fn/bootstrap | cut -f1) ($$env)"; \
		rm -f $(DIST_DIR)/$$fn.zip; \
		zip -j $(DIST_DIR)/$$fn.zip target/lambda/$$fn/bootstrap $$env; \
	done

# --- deploy just the api function (skips ws/cron, still WIP) ---

build-api:
	cargo lambda build -p backend --release --arm64 $(FEATURES) --bin api

package-api: build-api
	mkdir -p $(DIST_DIR)
	@echo "unzipped size api: $$(du -h target/lambda/api/bootstrap | cut -f1)"
	rm -f $(DIST_DIR)/api.zip
	zip -j $(DIST_DIR)/api.zip target/lambda/api/bootstrap .env.api

# for now deprecated because the migration from direct aws lambda logic to axum-server
# deploy-api: package-api
# 	cd terraform && terraform apply -target=aws_lambda_function.api -auto-approve $(TF_VARS)

# --- standalone binary deploy (e.g. diffusion-agent on EC2) ---

deploy-bin:
	cargo build -p backend --release --features datatransfer,remote-ec2 --bin diffusion-agent
	SANITY_RUN=true ENV_PATH="./.env.diffusion" ./target/release/diffusion-agent
	aws s3 cp .env.diffusion $(S3_BIN_BUCKET)/.env.diffusion
	aws s3 cp target/release/diffusion-agent $(S3_BIN_BUCKET)/diffusion-agent

# --- frontend (Leptos CSR, requires: cargo install trunk) ---

frontend-serve:
	cd crates/frontend && trunk serve

# Build-time config (API base, bucket names) comes from .env.frontend (or
# .env.frontend.example when the former is absent, e.g. CI), read by
# crates/frontend/build.rs and baked into the wasm. A process env var of
# the same name overrides the file — here PLAP_API_BASE is set from terraform's
# api_url output when the stack is applied, so a shipped bundle targets the deployed API.
frontend-build:
	cd crates/frontend && $(if $(strip $(API_URL)),PLAP_API_BASE="$(API_URL)" )trunk build --release

# Upload dist/ to S3 and bust the CloudFront cache.
# Hashed JS/WASM get a 1-year immutable cache; index.html is never cached.
frontend-deploy: frontend-build
	@test -n "$(strip $(API_URL))" || { echo "no api_url output — the bundle would target localhost; 'cd terraform && terraform apply $(TF_VARS)' first"; exit 1; }
	@test -n "$(FRONTEND_BUCKET)" || { echo "no frontend_bucket output — run 'make tf-init && cd terraform && terraform apply $(TF_VARS)' first"; exit 1; }
	aws s3 sync crates/frontend/dist/ s3://$(FRONTEND_BUCKET)/ --delete \
		--exclude ".stage/*" --exclude "index.html" \
		--cache-control "public,max-age=31536000,immutable"
	aws s3 cp crates/frontend/dist/index.html s3://$(FRONTEND_BUCKET)/index.html \
		--cache-control "no-cache"
	aws cloudfront create-invalidation \
		--distribution-id $(FRONTEND_DISTRIBUTION_ID) --paths "/*"

# --- local run ---
# Starts a local Lambda API server on http://localhost:9000
# Then use make invoke-* or curl/wscat to test

# existing local DynamoDB container (host port 40000 -> 8000)
dynamo:
	docker start dynamodb-local && dynamodb-admin --dynamo-endpoint=http://localhost:40000

# create the tables from terraform/dynamodb.tf in the local container (run `make dynamo` first)
dynamo-tables:
	cd terraform/local && terraform init && terraform apply

run-api:
	ENV_PATH=.env.api.localhost cargo run -p backend --bin api

# same as run-api, but rebuilds and restarts on source changes (requires: cargo install cargo-watch)
watch-api:
	ENV_PATH=.env.api.localhost cargo watch -w crates -w Cargo.toml -x 'run -p backend --bin api'

invoke-api:
	@cargo lambda invoke api --invoke-address 127.0.0.1 --data-file /tmp/api.json

invoke-ws:
	@cargo lambda invoke ws --invoke-address 127.0.0.1 --data-file events/ws.json

run-cron:
	cargo lambda watch --invoke-address 127.0.0.1 --env-file .env.production

invoke-cron:
	@cargo lambda invoke cron --invoke-address 127.0.0.1 --data-file events/cron.json

# --- terraform ---
# Run `make tf-init` once before first deploy

tf-init:
	cd terraform && terraform init

plan deploy deploy-api: FEATURES := --features remote-ec2

plan: package
	cd terraform && terraform plan $(TF_VARS)

deploy: package
	cd terraform && terraform apply -auto-approve $(TF_VARS)

# --- cleanup ---

clean:
	cargo clean
	rm -rf $(DIST_DIR)
