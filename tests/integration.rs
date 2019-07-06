//! End-to-end integration tests that exercise the public library API against
//! real child processes. These use the host shell so they run on both Windows
//! and Unix CI.

use litmus::report::investigate;
use litmus::{render_text, Scenario, Verdict};

/// Build a scenario whose command is a portable shell invocation.
fn scenario_with_command(header: &str, shell_cmd: &str) -> Scenario {
    let command_line = if cfg!(windows) {
        format!("command = cmd /C \"{shell_cmd}\"")
    } else {
        format!("command = sh -c \"{shell_cmd}\"")
    };
    let text = format!("{header}\n[run]\n{command_line}");
    Scenario::parse(&text).expect("scenario should parse")
}

#[test]
fn deterministic_command_is_stable() {
    let s = scenario_with_command("name = det\n#", "echo litmus");
    // Override runs via the struct since our header does not set it.
    let mut s = s;
    s.runs = 6;
    s.expect = litmus::Expectation::Deterministic;

    let report = investigate(&s).expect("run should succeed");
    assert_eq!(report.distinct_outcomes, 1);
    assert_eq!(report.verdict, Verdict::Stable);
    assert!(!report.is_failure());
