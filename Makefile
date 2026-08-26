STAGE     ?= production
FUNCTIONS := api ws cron
DIST_DIR  := dist

S3_BIN_BUCKET := s3://virginia-ramadoka/bin

TF_VARS := -var="stage=$(STAGE)"

.PHONY: all verify check lint fmt test typos build package plan deploy run \
        invoke-api invoke-ws invoke-cron tf-init clean deploy-bin \
        build-api package-api deploy-api frontend-serve frontend-build

all: verify build

# --- code quality ---

verify: check lint test typos

check:
	cargo check -p backend

lint:
	cargo clippy -p backend --features datatransfer -- -D warnings -A dead_code \
		-A clippy::let_and_return -A clippy::bool_comparison -A clippy::upper_case_acronyms

fix:
	cargo fix -p backend --allow-dirty --allow-staged --features datatransfer

fmt:
	cargo fmt

test:
	cargo test -p backend --features="datatransfer"

typos:
	typos

coverage:
	cargo llvm-cov -p backend --features datatransfer --html && open target/llvm-cov/html/index.html

# --- build & package ---
# Requires: cargo-lambda (cargo install cargo-lambda)

build:
	cargo lambda build -p backend --release --arm64 --bin api --bin ws --bin cron

package: build
	mkdir -p $(DIST_DIR)
	@for fn in $(FUNCTIONS); do \
		echo "unzipped size $$fn: $$(du -h target/lambda/$$fn/bootstrap | cut -f1)"; \
		zip -j $(DIST_DIR)/$$fn.zip target/lambda/$$fn/bootstrap .env.production; \
	done

# --- deploy just the api function (skips ws/cron, still WIP) ---

build-api:
	cargo lambda build -p backend --release --arm64 --bin api

package-api: build-api
	mkdir -p $(DIST_DIR)
	@echo "unzipped size api: $$(du -h target/lambda/api/bootstrap | cut -f1)"
	zip -j $(DIST_DIR)/api.zip target/lambda/api/bootstrap .env.production

deploy-api: package-api
	cd terraform && terraform apply -target=aws_lambda_function.api -auto-approve $(TF_VARS)

# --- standalone binary deploy (e.g. diffusion-agent on EC2) ---

deploy-bin:
	cargo build -p backend --release --features datatransfer --bin diffusion-agent
	SANITY_RUN=true ENV_PATH="./.env.diffusion" ./target/release/diffusion-agent
	aws s3 cp .env.diffusion $(S3_BIN_BUCKET)/.env.diffusion
	aws s3 cp target/release/diffusion-agent $(S3_BIN_BUCKET)/diffusion-agent

# --- frontend (Leptos CSR, requires: cargo install trunk) ---

frontend-serve:
	cd crates/frontend && trunk serve

frontend-build:
	cd crates/frontend && trunk build --release

# --- local run ---
# Starts a local Lambda API server on http://localhost:9000
# Then use make invoke-* or curl/wscat to test

run-api:
	cargo lambda watch --invoke-address 127.0.0.1 --env-file .env.api

invoke-api:
	@cargo lambda invoke api --invoke-address 127.0.0.1 --data-file /tmp/api.json

invoke-ws:
	@cargo lambda invoke ws --invoke-address 127.0.0.1 --data-file events/ws.json

invoke-cron:
	@cargo lambda invoke cron --invoke-address 127.0.0.1 --data-file events/cron.json

# --- terraform ---
# Run `make tf-init` once before first deploy

tf-init:
	cd terraform && terraform init

plan: package
	cd terraform && terraform plan $(TF_VARS)

deploy: package
	cd terraform && terraform apply -auto-approve $(TF_VARS)

# --- cleanup ---

clean:
	cargo clean
	rm -rf $(DIST_DIR)
