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

