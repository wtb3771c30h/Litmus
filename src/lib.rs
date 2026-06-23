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
                total_lines - sample_lines,
                if total_lines - sample_lines == 1 {
                    ""
                } else {
                    "s"
                },
                c.reset
            ));
        }
    }
}

/// Build a unicode bar of `width` cells representing `count/total`.
fn bar(count: u32, total: u32, width: usize) -> String {
    if total == 0 {
        return " ".repeat(width);
    }
    let filled = ((count as f64 / total as f64) * width as f64).round() as usize;
    let filled = filled.min(width);
    let mut s = String::with_capacity(width);
    for _ in 0..filled {
        s.push('█');
    }
    for _ in filled..width {
        s.push('░');
    }
    s
}

/// Collapse a sorted-ish index list into compact ranges, e.g. `0-3, 5, 7-9`.
fn summarize_indices(indices: &[u32]) -> String {
    if indices.is_empty() {
        return "(none)".to_string();
    }
    let mut sorted = indices.to_vec();
    sorted.sort_unstable();
    let mut parts: Vec<String> = Vec::new();
    let mut start = sorted[0];
    let mut prev = sorted[0];
    for &v in &sorted[1..] {
        if v == prev + 1 {
            prev = v;
            continue;
        }
        parts.push(fmt_range(start, prev));
        start = v;
        prev = v;
    }
    parts.push(fmt_range(start, prev));
    parts.join(", ")
}

fn fmt_range(start: u32, end: u32) -> String {
    if start == end {
        start.to_string()
    } else {
        format!("{start}-{end}")
    }
}

/// ANSI palette; every field is empty when color is disabled.
struct Palette {
    reset: &'static str,
    bold: &'static str,
    dim: &'static str,
    red: &'static str,
    green: &'static str,
    yellow: &'static str,
}

impl Palette {
    fn new(color: bool) -> Palette {
        if color {
            Palette {
                reset: "\x1b[0m",
                bold: "\x1b[1m",
                dim: "\x1b[2m",
                red: "\x1b[31m",
                green: "\x1b[32m",
                yellow: "\x1b[33m",
            }
        } else {
            Palette {
                reset: "",
                bold: "",
                dim: "",
                red: "",
                green: "",
                yellow: "",
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarize_ranges() {
        assert_eq!(summarize_indices(&[0, 1, 2, 3]), "0-3");
        assert_eq!(summarize_indices(&[0, 2, 3, 5]), "0, 2-3, 5");
        assert_eq!(summarize_indices(&[7]), "7");
        assert_eq!(summarize_indices(&[]), "(none)");
        // Unsorted input is handled.
        assert_eq!(summarize_indices(&[3, 0, 1, 2]), "0-3");
    }

    #[test]
    fn bar_fills_proportionally() {
        assert_eq!(bar(0, 10, 10), "░░░░░░░░░░");
        assert_eq!(bar(10, 10, 10), "██████████");
        assert_eq!(bar(5, 10, 10), "█████░░░░░");
        assert_eq!(bar(1, 0, 4), "    ");
    }

    #[test]
    fn text_report_mentions_verdict_and_command() {
        let s = Scenario::parse("name = demo\n[run]\ncommand = echo hi\nexpect = deterministic")
            .unwrap();
        let outcomes = vec![report::Outcome {
            run_index: 0,
            seed: 0,
            fingerprint: "abc".into(),
            exit_code: Some(0),
            timed_out: false,
            duration_ms: 1,
            canonical: "hi\n".into(),
        }];
        let r = report::build_report(&s, outcomes);
        let text = render_text(&r, false, 3);
        assert!(text.contains("demo"));
        assert!(text.contains("echo hi"));
        assert!(text.contains("STABLE"));
    }
}
