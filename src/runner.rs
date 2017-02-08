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
