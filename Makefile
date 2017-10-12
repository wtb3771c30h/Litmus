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
