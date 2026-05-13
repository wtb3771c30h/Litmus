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
    (`stable` / `diverged` / `expectation_violated`), and honors the scenario's
    `expect = deterministic | flaky` assertion.
  - Commands: `run`, `check` (CI-friendly non-zero exit on violation), `emit`
    (JSON evidence), `explain`, `version`, `help`.
  - Human-readable, colorized text report with proportional outcome bars and
    compact run-index ranges.
  - Machine-readable JSON evidence (`schema: litmus/evidence@1`) via a
    hand-written, dependency-free JSON writer.
- **TypeScript tooling (`litmus-report`)**, dependency-free:
  - Strongly-typed evidence parser/validator.
  - Self-contained HTML report renderer (inlined CSS, no external assets) and
    a plain-text renderer.
  - Programmatic scenario **authoring helper** that serializes typed specs to
    `.litmus` files.
  - `litmus-report html|text <evidence.json>` CLI.
- **Example scenarios** (cross-platform via Node): deterministic echo,
  timestamp-normalized, genuinely flaky random outcome, seed-controlled
  determinism, and a set-ordering flaky/fixed pair.
- **Docs**: rich README with Mermaid diagrams and a live demo transcript,
  `docs/SCENARIO.md` reference.
- **Project infrastructure**: `Makefile`, GitHub Actions CI (Rust + TypeScript
  on Linux/macOS/Windows), `.gitignore`, MIT `LICENSE`.
- **Tests**: 43 Rust unit tests, 4 Rust integration tests exercising real child
  processes, and 14 TypeScript tests via the built-in `node:test` runner.

[0.1.0]: https://github.com/example/litmus/releases/tag/v0.1.0
