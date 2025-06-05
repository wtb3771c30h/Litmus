.PHONY: all build test fmt fmt-check lint clippy ts-build ts-test tools demo emit clean help

# Default: build and test everything.
all: build ts-build test ts-test

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

## TypeScript ----------------------------------------------------------------

ts-build: ## Type-check and compile the TypeScript tooling.
	cd tools && npx tsc -p tsconfig.json

ts-test: ts-build ## Run the TypeScript test suite.
	cd tools && node --test dist/litmus.test.js

tools: ts-build ## Alias for ts-build.

## Demos ---------------------------------------------------------------------

demo: build ## Run every example scenario with the release binary.
	./target/release/litmus run scenarios/deterministic-echo.litmus
	./target/release/litmus run scenarios/normalized-timestamp.litmus
	./target/release/litmus run scenarios/flaky-random-outcome.litmus
	./target/release/litmus run scenarios/set-order-flaky.litmus
	./target/release/litmus run scenarios/set-order-fixed.litmus

emit: build ts-build ## Emit evidence and render an HTML report end-to-end.
	./target/release/litmus emit scenarios/flaky-random-outcome.litmus --out examples/evidence.local.json
	cd tools && node dist/cli.js html ../examples/evidence.local.json --out ../examples/report.html

clean: ## Remove build artifacts.
	cargo clean
	cd tools && node -e "require('fs').rmSync('dist',{recursive:true,force:true})"

help: ## Show this help.
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-14s\033[0m %s\n", $$1, $$2}'
