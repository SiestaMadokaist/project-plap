STAGE     ?= production
FUNCTIONS := api ws cron
DIST_DIR  := dist

S3_BIN_BUCKET := s3://virginia-ramadoka/bin

TF_VARS := -var="stage=$(STAGE)"

.PHONY: all verify fmt test build package plan deploy run \
        invoke-api invoke-ws invoke-cron tf-init clean deploy-bin \
        build-api package-api deploy-api

all: verify test build

# --- code quality ---

verify:
	cargo check
	cargo clippy -- -D warnings

fmt:
	cargo fmt

test:
	cargo test --features="datatransfer"

coverage:
	cargo llvm-cov --features datatransfer --html && open target/llvm-cov/html/index.html

# --- build & package ---
# Requires: cargo-lambda (cargo install cargo-lambda)

build:
	cargo lambda build --release --arm64 --bin api --bin ws --bin cron

package: build
	mkdir -p $(DIST_DIR)
	@for fn in $(FUNCTIONS); do \
		echo "unzipped size $$fn: $$(du -h target/lambda/$$fn/bootstrap | cut -f1)"; \
		zip -j $(DIST_DIR)/$$fn.zip target/lambda/$$fn/bootstrap .env.production; \
	done

# --- deploy just the api function (skips ws/cron, still WIP) ---

build-api:
	cargo lambda build --release --arm64 --bin api

package-api: build-api
	mkdir -p $(DIST_DIR)
	@echo "unzipped size api: $$(du -h target/lambda/api/bootstrap | cut -f1)"
	zip -j $(DIST_DIR)/api.zip target/lambda/api/bootstrap .env.production

deploy-api: package-api
	cd terraform && terraform apply -target=aws_lambda_function.api -auto-approve $(TF_VARS)

# --- standalone binary deploy (e.g. diffusion-agent on EC2) ---

deploy-bin:
	cargo build --release --features datatransfer --bin diffusion-agent
	SANITY_RUN=true ENV_PATH="./.env.diffusion" ./target/release/diffusion-agent
	aws s3 cp target/release/diffusion-agent $(S3_BIN_BUCKET)/diffusion-agent
	aws s3 cp .env.diffusion $(S3_BIN_BUCKET)/.env.diffusion

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
