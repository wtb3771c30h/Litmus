//! `litmus` command-line interface.
//!
//! Usage:
//!
//! ```text
//! litmus run <scenario.litmus> [options]
//! litmus check <scenario.litmus> [options]   # like run, exits non-zero on violation
//! litmus emit <scenario.litmus> [--out FILE] # run and write JSON evidence
//! litmus explain <scenario.litmus>           # parse and print the scenario config
//! litmus version | help
//! ```
//!
//! Options:
//!   --runs N            override the scenario's run count
//!   --timeout-ms N      override the per-run timeout
