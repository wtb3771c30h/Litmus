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
    cmd.stderr(Stdio::piped());

    let start = Instant::now();
    let mut child = cmd.spawn()?;

    // Drain stdout/stderr on separate threads so a large output cannot deadlock
    // against a full pipe buffer while we wait.
    let mut out_handle = child.stdout.take();
    let mut err_handle = child.stderr.take();

    let (out_tx, out_rx) = mpsc::channel();
    let (err_tx, err_rx) = mpsc::channel();

    let out_thread = thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(ref mut h) = out_handle {
            let _ = h.read_to_end(&mut buf);
        }
        let _ = out_tx.send(buf);
    });
    let err_thread = thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(ref mut h) = err_handle {
            let _ = h.read_to_end(&mut buf);
        }
        let _ = err_tx.send(buf);
    });

    let mut timed_out = false;
    let status = match cfg.timeout {
        None => child.wait()?,
        Some(limit) => loop {
            match child.try_wait()? {
                Some(status) => break status,
                None => {
                    if start.elapsed() >= limit {
                        let _ = child.kill();
                        timed_out = true;
                        break child.wait()?;
                    }
                    thread::sleep(poll_interval(limit));
                }
            }
        },
    };

    let duration_ms = start.elapsed().as_millis();

    // Threads finish once the pipes close (which happens when the child exits
    // or is killed). Join them and collect the buffers.
    let stdout_bytes = out_rx.recv().unwrap_or_default();
    let stderr_bytes = err_rx.recv().unwrap_or_default();
    let _ = out_thread.join();
    let _ = err_thread.join();

    Ok(RunOutput {
        stdout: String::from_utf8_lossy(&stdout_bytes).into_owned(),
        stderr: String::from_utf8_lossy(&stderr_bytes).into_owned(),
        exit_code: status.code(),
        timed_out,
        duration_ms,
    })
}

/// Choose a polling interval: fine-grained for short timeouts, coarser for long
/// ones, clamped to a sane range to avoid busy-waiting.
fn poll_interval(limit: Duration) -> Duration {
    let ms = (limit.as_millis() / 50).clamp(2, 25) as u64;
    Duration::from_millis(ms)
}

/// Build the ordered env list for a run. When `shuffle` is set, the insertion
/// order is permuted deterministically (driven by the caller's RNG) so that
/// programs sensitive to environment ordering can be surfaced.
pub fn ordered_env(env: &BTreeMap<String, String>, order: &[usize]) -> Vec<(String, String)> {
    let pairs: Vec<(String, String)> = env.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    if order.len() == pairs.len() {
        order.iter().map(|&i| pairs[i].clone()).collect()
    } else {
        pairs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shell(cmd: &str) -> Vec<String> {
        if cfg!(windows) {
            vec!["cmd".into(), "/C".into(), cmd.into()]
        } else {
            vec!["sh".into(), "-c".into(), cmd.into()]
        }
    }

    #[test]
    fn captures_stdout_and_exit_code() {
        let command = shell("echo hello");
        let cfg = RunConfig {
            command: &command,
            env: &[],
            workdir: None,
            timeout: None,
        };
        let out = run_once(&cfg).unwrap();
        assert!(out.stdout.contains("hello"));
        assert_eq!(out.exit_code, Some(0));
        assert!(!out.timed_out);
    }

    #[test]
    fn captures_nonzero_exit() {
        let command = shell("exit 3");
        let cfg = RunConfig {
            command: &command,
            env: &[],
            workdir: None,
            timeout: None,
        };
        let out = run_once(&cfg).unwrap();
        assert_eq!(out.exit_code, Some(3));
    }

    #[test]
    fn enforces_timeout() {
        // Sleep well beyond the timeout; expect it to be killed.
        let command = if cfg!(windows) {
            // ping delays ~1s per count; 5 counts ~4s of waiting.
            shell("ping -n 5 127.0.0.1 >NUL")
        } else {
            shell("sleep 3")
        };
        let cfg = RunConfig {
            command: &command,
            env: &[],
            workdir: None,
            timeout: Some(Duration::from_millis(300)),
        };
        let out = run_once(&cfg).unwrap();
        assert!(out.timed_out, "expected the run to time out");
    }

    #[test]
    fn spawn_error_for_missing_program() {
        let command = vec!["this-program-does-not-exist-xyz".to_string()];
        let cfg = RunConfig {
            command: &command,
            env: &[],
            workdir: None,
            timeout: None,
        };
        assert!(run_once(&cfg).is_err());
    }

    #[test]
    fn ordered_env_respects_order() {
        let mut env = BTreeMap::new();
        env.insert("A".to_string(), "1".to_string());
        env.insert("B".to_string(), "2".to_string());
        env.insert("C".to_string(), "3".to_string());
        let out = ordered_env(&env, &[2, 0, 1]);
        assert_eq!(out[0].0, "C");
        assert_eq!(out[1].0, "A");
        assert_eq!(out[2].0, "B");
    }
}
