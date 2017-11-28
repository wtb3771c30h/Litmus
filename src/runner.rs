//! Command execution with an optional per-run timeout.
//!
//! The standard library has no built-in "wait with timeout", so we implement
//! one with a helper thread that reads the child's stdout/stderr and a polling
//! loop that enforces the deadline. On timeout we kill the child and report it.

use std::collections::BTreeMap;
use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

/// The captured result of a single command execution.
#[derive(Debug, Clone)]
pub struct RunOutput {
    pub stdout: String,
    pub stderr: String,
    /// Exit code, or `None` if the process was killed by a signal / timeout.
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub duration_ms: u128,
}

/// Configuration for a single execution.
#[derive(Debug, Clone)]
pub struct RunConfig<'a> {
    pub command: &'a [String],
    /// Environment entries to add, in the order they should be applied.
    pub env: &'a [(String, String)],
    pub workdir: Option<&'a str>,
    pub timeout: Option<Duration>,
}

/// Execute the command once and capture its output.
///
/// Returns an `Err` only when the process could not be spawned at all
/// (e.g. the program does not exist). A non-zero exit or a timeout is a
/// successful *observation* and is reported inside `RunOutput`.
pub fn run_once(cfg: &RunConfig) -> std::io::Result<RunOutput> {
    let (program, args) = cfg
        .command
        .split_first()
        .expect("command validated non-empty during parsing");

    let mut cmd = Command::new(program);
    cmd.args(args);
    for (k, v) in cfg.env {
        cmd.env(k, v);
    }
    if let Some(dir) = cfg.workdir {
        cmd.current_dir(dir);
    }
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
