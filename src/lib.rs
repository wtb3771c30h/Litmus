//! # litmus
//!
//! A nondeterminism / flaky-test laboratory.
//!
//! `litmus` reads a [scenario](scenario) file describing a command to run, then
//! executes it repeatedly under controlled conditions (per-run seeds, injected
//! environment variables, optional environment-order shuffling, and per-run
//! timeouts). Each run's output is [normalized](normalize) to strip legitimate
//! noise (timestamps, addresses, temp paths) and [fingerprinted](hash). Runs
//! are then grouped by fingerprint to [report](report) whether the command is
//! stable or divergent, and to emit machine-readable JSON evidence.
//!
//! The entire crate uses only the Rust standard library.

pub mod hash;
pub mod json;
pub mod normalize;
pub mod report;
pub mod runner;
