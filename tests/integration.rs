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
    assert_eq!(report.total_runs, 6);
}

#[test]
fn divergent_exit_codes_are_detected() {
    // A command that alternates exit code based on the injected LITMUS_RUN.
    // Even with identical stdout, the differing exit status must diverge.
    let shell_cmd = if cfg!(windows) {
        // cmd: use modulo via set /a on LITMUS_RUN.
        "set /a r=%LITMUS_RUN% %% 2 & exit /b %r%"
    } else {
        "exit $(( LITMUS_RUN % 2 ))"
    };
    let mut s = scenario_with_command("name = exitdiverge\n#", shell_cmd);
    s.runs = 6;
    s.expect = litmus::Expectation::Unspecified;

    let report = investigate(&s).expect("run should succeed");
    assert!(
        report.distinct_outcomes >= 2,
        "expected divergence from alternating exit codes, got {} distinct",
        report.distinct_outcomes
    );
    assert_eq!(report.verdict, Verdict::Diverged);
}

#[test]
fn deterministic_expectation_violated_when_output_varies() {
    // Print the run index -> every run differs -> violates a determinism claim.
    let shell_cmd = if cfg!(windows) {
        "echo %LITMUS_RUN%"
    } else {
        "echo $LITMUS_RUN"
    };
    let mut s = scenario_with_command("name = violate\n#", shell_cmd);
    s.runs = 4;
    s.expect = litmus::Expectation::Deterministic;

    let report = investigate(&s).expect("run should succeed");
    assert_eq!(report.verdict, Verdict::ExpectationViolated);
    assert!(report.is_failure());

    // The text report should surface the violation.
    let text = render_text(&report, false, 2);
    assert!(text.contains("EXPECTATION VIOLATED"), "report was:\n{text}");
}

#[test]
fn json_evidence_round_trips_shape() {
    let mut s = scenario_with_command("name = evidence\n#", "echo ok");
    s.runs = 3;
    let report = investigate(&s).expect("run should succeed");
    let json = report.to_json().to_compact();

    assert!(json.contains(r#""schema":"litmus/evidence@1""#));
    assert!(json.contains(r#""scenario":"evidence""#));
    assert!(json.contains(r#""total_runs":3"#));
    assert!(json.contains(r#""verdict":"stable""#));
    // Every outcome has a fingerprint field.
    assert!(json.contains(r#""fingerprint":"#));
}
