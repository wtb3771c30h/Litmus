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
//!   --seed N            override the base seed
//!   --json              print JSON evidence to stdout instead of a text report
//!   --out FILE          write JSON evidence to FILE (implies emit)
//!   --no-color          disable ANSI colors
//!   --sample-lines N    lines of sample output to show per group (default 4)

use std::process::ExitCode;

use litmus::report::investigate;
use litmus::{render_text, Scenario};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("litmus: error: {e}");
            ExitCode::from(2)
        }
    }
}

struct Options {
    scenario_path: String,
    runs: Option<u32>,
    timeout_ms: Option<u64>,
    seed: Option<u64>,
    json: bool,
    out: Option<String>,
    color: bool,
    sample_lines: usize,
}

enum Mode {
    Run,
    Check,
    Emit,
    Explain,
}

fn run(args: &[String]) -> Result<ExitCode, String> {
    let command = match args.first().map(String::as_str) {
        None | Some("help") | Some("-h") | Some("--help") => {
            print_help();
            return Ok(ExitCode::SUCCESS);
        }
        Some("version") | Some("-V") | Some("--version") => {
            println!("litmus {}", env!("CARGO_PKG_VERSION"));
            return Ok(ExitCode::SUCCESS);
        }
        Some("run") => Mode::Run,
        Some("check") => Mode::Check,
        Some("emit") => Mode::Emit,
        Some("explain") => Mode::Explain,
        Some(other) => return Err(format!("unknown command {other:?} (try `litmus help`)")),
    };

    let opts = parse_options(&args[1..])?;

    let source = std::fs::read_to_string(&opts.scenario_path)
        .map_err(|e| format!("cannot read {}: {e}", opts.scenario_path))?;
