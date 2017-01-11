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
