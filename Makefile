STAGE     ?= production
FUNCTIONS := api ws cron
DIST_DIR  := dist

TF_VARS := -var="stage=$(STAGE)"

.PHONY: all verify fmt test build package plan deploy run \
        invoke-api invoke-ws invoke-cron tf-init clean

all: verify test build

# --- code quality ---

verify:
	typos
	cargo check
	cargo clippy -- -D warnings

fmt:
	cargo fmt

test:
	cargo test

# --- build & package ---
# Requires: cargo-lambda (cargo install cargo-lambda)

build:
	cargo lambda build --release --arm64

package: build
	mkdir -p $(DIST_DIR)
	@for fn in $(FUNCTIONS); do \
		echo "unzipped size $$fn: $$(du -h target/lambda/$$fn/bootstrap | cut -f1)"; \
		zip -j $(DIST_DIR)/$$fn.zip target/lambda/$$fn/bootstrap .env.production; \
	done

# --- local run ---
# Starts a local Lambda API server on http://localhost:9000
# Then use make invoke-* or curl/wscat to test

run:
	cargo lambda watch --env-file .env.local

invoke-api:
	cargo lambda invoke api --data-file events/api.json

invoke-ws:
	cargo lambda invoke ws --data-file events/ws.json

invoke-cron:
	cargo lambda invoke cron --data-file events/cron.json

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
