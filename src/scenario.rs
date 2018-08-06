//! Scenario file parsing.
//!
//! A scenario file uses a tiny, dependency-free `key = value` format with
//! `[section]` headers. It is deliberately simple so that authoring scenarios
//! by hand (or from the TypeScript helper) requires no external tooling.
//!
//! Grammar (informal):
//!
//! ```text
//! # comment line (also ';' comments)
//! name = my scenario            # top-level scalar
//!
//! [run]
//! command = echo hello          # required: the program + args to execute
//! runs = 20                     # optional: number of repetitions (default 10)
//! timeout_ms = 5000             # optional: per-run timeout in milliseconds
//! shuffle_env = true            # optional: randomize env-var insertion order
//! seed = 12345                  # optional: base seed for deterministic jitter
//! workdir = .                   # optional: working directory
//! expect = deterministic        # optional: deterministic | flaky (assertion)
//!
//! [env]
//! GREETING = hello              # extra environment variables (repeatable)
//!
//! [normalize]
//! strip_timestamps = true
//! strip_hex = true
//! strip_temp_paths = true
//! trim_trailing_ws = true
//! sort_lines = false
//! ```
//!
//! Lists (such as multiple `arg =` entries) accumulate in order.

use std::collections::BTreeMap;
use std::fmt;

/// What we assert about a scenario's stability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expectation {
    /// Every run must produce the same normalized fingerprint.
    Deterministic,
    /// Runs are expected to diverge (at least two distinct fingerprints).
    Flaky,
    /// No assertion; just report what happens.
    Unspecified,
}

impl Expectation {
    fn parse(s: &str) -> Result<Expectation, ParseError> {
        match s.trim().to_ascii_lowercase().as_str() {
            "deterministic" | "stable" => Ok(Expectation::Deterministic),
            "flaky" | "nondeterministic" => Ok(Expectation::Flaky),
            "" | "any" | "unspecified" => Ok(Expectation::Unspecified),
            other => Err(ParseError::new(format!("unknown expectation: {other:?}"))),
        }
    }
}

impl fmt::Display for Expectation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Expectation::Deterministic => "deterministic",
            Expectation::Flaky => "flaky",
            Expectation::Unspecified => "unspecified",
        };
        f.write_str(s)
    }
}

/// Options controlling output normalization before fingerprinting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizeOptions {
    pub strip_timestamps: bool,
    pub strip_hex: bool,
    pub strip_temp_paths: bool,
    pub trim_trailing_ws: bool,
    pub sort_lines: bool,
    /// Drop lines matching any of these literal substrings.
    pub drop_substrings: Vec<String>,
}

impl Default for NormalizeOptions {
    fn default() -> Self {
        NormalizeOptions {
            strip_timestamps: true,
            strip_hex: true,
            strip_temp_paths: true,
            trim_trailing_ws: true,
            sort_lines: false,
            drop_substrings: Vec::new(),
        }
    }
}

/// A fully parsed scenario, ready to be executed by the runner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scenario {
    pub name: String,
    pub description: String,
    /// The command: first element is the program, rest are arguments.
    pub command: Vec<String>,
    pub runs: u32,
    pub timeout_ms: Option<u64>,
    pub shuffle_env: bool,
    pub seed: u64,
    pub workdir: Option<String>,
    pub expect: Expectation,
    pub env: BTreeMap<String, String>,
    pub normalize: NormalizeOptions,
}

impl Scenario {
    /// Parse a scenario from raw text.
    pub fn parse(input: &str) -> Result<Scenario, ParseError> {
        let mut name = String::new();
        let mut description = String::new();
        let mut command: Vec<String> = Vec::new();
        let mut runs: u32 = 10;
        let mut timeout_ms: Option<u64> = None;
        let mut shuffle_env = false;
        let mut seed: u64 = 0;
        let mut workdir: Option<String> = None;
        let mut expect = Expectation::Unspecified;
        let mut env: BTreeMap<String, String> = BTreeMap::new();
        let mut normalize = NormalizeOptions::default();

        let mut section = String::new();

        for (idx, raw_line) in input.lines().enumerate() {
            let lineno = idx + 1;
            let line = strip_comment(raw_line).trim();
            if line.is_empty() {
                continue;
            }
            if let Some(sec) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                section = sec.trim().to_ascii_lowercase();
                continue;
            }
            let (key, value) = split_kv(line).ok_or_else(|| {
                ParseError::at(lineno, format!("expected `key = value`, got {line:?}"))
            })?;
            let key = key.to_ascii_lowercase();

            match section.as_str() {
                "" => match key.as_str() {
                    "name" => name = value.to_string(),
                    "description" => description = value.to_string(),
                    other => {
                        return Err(ParseError::at(
                            lineno,
                            format!("unknown top-level key: {other:?}"),
                        ))
                    }
                },
                "run" => match key.as_str() {
