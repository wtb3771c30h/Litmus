# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-08-31

### Added

- **Rust CLI (`litmus`)**, standard-library only:
  - Scenario file format (`.litmus`) with `[run]`, `[env]`, and `[normalize]`
    sections, tokenized command parsing, and helpful line-numbered errors.
  - Repeated execution of a target command with:
    - per-run derived seeds exposed as `LITMUS_SEED` / `LITMUS_RUN`,
    - injected environment variables with optional deterministic order
      shuffling (`shuffle_env`),
    - per-run timeouts enforced via a polling wait + child kill.
  - Output **normalization** passes: timestamp masking (`<ts>`), hex/address
    masking (`<hex>`), temp-path masking (`<tmp>`, Unix + Windows), trailing
    whitespace trimming, line sorting, and substring dropping.
  - **Fingerprinting** with a stable FNV-1a/64 hash over the normalized
    stdout + stderr + exit status + timeout flag.
  - **Divergence analysis**: groups identical outcomes, computes a verdict
