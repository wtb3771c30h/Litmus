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
    if report.timeouts > 0 {
        out.push_str(&format!(
            "  {}timeouts     : {}{}\n",
            c.yellow, report.timeouts, c.reset
        ));
    }

    let (label, color_code) = match report.verdict {
        Verdict::Stable => ("STABLE", c.green),
        Verdict::Diverged => ("DIVERGED", c.yellow),
        Verdict::ExpectationViolated => ("EXPECTATION VIOLATED", c.red),
    };
    out.push_str(&format!(
        "  verdict      : {}{}{}\n",
        color_code, label, c.reset
    ));

    out.push('\n');
    out.push_str(&format!("  {}outcome groups:{}\n", c.bold, c.reset));
    for (i, g) in report.groups.iter().enumerate() {
        render_group(&mut out, &c, i, g, report.total_runs, sample_lines);
    }

    out
}

fn render_group(
    out: &mut String,
    c: &Palette,
    index: usize,
    g: &OutcomeGroup,
    total: u32,
    sample_lines: usize,
) {
    let pct = if total > 0 {
        (g.count as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    let bar = bar(g.count, total, 24);
    out.push_str(&format!(
        "  [{}] {} {:>3}/{:<3} ({:5.1}%)  fp={}\n",
        index, bar, g.count, total, pct, g.fingerprint
    ));
    out.push_str(&format!(
        "      runs: {}\n",
        summarize_indices(&g.run_indices)
    ));
    if sample_lines > 0 {
        let shown: Vec<&str> = g.sample.lines().take(sample_lines).collect();
        for line in &shown {
            out.push_str(&format!("      {}| {}{}\n", c.dim, line, c.reset));
        }
        let total_lines = g.sample.lines().count();
        if total_lines > sample_lines {
            out.push_str(&format!(
                "      {}| ... ({} more line{}){}\n",
                c.dim,
