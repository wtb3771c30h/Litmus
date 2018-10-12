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
    if !report.description.is_empty() {
        out.push_str(&format!("  {}\n", report.description));
    }
    out.push_str(&format!("  command      : {}\n", report.command.join(" ")));
    out.push_str(&format!(
        "  runs         : {}  ({} distinct outcome{})\n",
        report.total_runs,
        report.distinct_outcomes,
        if report.distinct_outcomes == 1 {
            ""
        } else {
            "s"
        }
    ));
    out.push_str(&format!("  expectation  : {}\n", report.expectation));
    out.push_str(&format!(
        "  duration     : {}ms .. {}ms\n",
        report.min_duration_ms, report.max_duration_ms
    ));
