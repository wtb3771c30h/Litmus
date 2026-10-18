.PHONY: all build test fmt fmt-check lint clippy demo emit clean help

# Default: build and test everything.
all: build test

## Rust ----------------------------------------------------------------------

build: ## Build the Rust CLI (release).
	cargo build --release

test: ## Run all Rust tests (unit + integration).
	cargo test

fmt: ## Format Rust code.
	cargo fmt

fmt-check: ## Check Rust formatting without modifying files.
	cargo fmt --check

clippy: ## Run clippy with warnings denied.
	cargo clippy --all-targets -- -D warnings

lint: fmt-check clippy ## Run all Rust lints.

## Demos ---------------------------------------------------------------------

demo: build ## Run every example scenario with the release binary.
	./target/release/litmus run scenarios/deterministic-echo.litmus
	./target/release/litmus run scenarios/normalized-timestamp.litmus
	./target/release/litmus run scenarios/flaky-random-outcome.litmus
	./target/release/litmus run scenarios/set-order-flaky.litmus
	./target/release/litmus run scenarios/set-order-fixed.litmus

emit: build ## Emit an evidence file for the flaky scenario.
	./target/release/litmus emit scenarios/flaky-random-outcome.litmus --out examples/evidence.local.json

clean: ## Remove build artifacts.
	cargo clean

help: ## Show this help.
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-14s\033[0m %s\n", $$1, $$2}'
