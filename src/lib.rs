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
pub mod scenario;

pub use report::{investigate, Report, Verdict};
pub use scenario::{Expectation, Scenario};

use report::OutcomeGroup;

/// Render a human-readable text report.
///
/// `color` toggles ANSI escapes; `sample_lines` bounds how many lines of a
/// divergent sample are shown per group (0 = none).
pub fn render_text(report: &Report, color: bool, sample_lines: usize) -> String {
    let mut out = String::new();
    let c = Palette::new(color);

    out.push_str(&format!(
        "{}litmus{} :: {}\n",
        c.bold, c.reset, report.scenario_name
    ));
