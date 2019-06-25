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
