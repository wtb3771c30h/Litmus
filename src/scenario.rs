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
                    "command" => command = tokenize_command(value)?,
                    "arg" => command.push(value.to_string()),
                    "runs" => {
                        runs = value.parse().map_err(|_| {
                            ParseError::at(lineno, format!("invalid runs: {value:?}"))
                        })?
                    }
                    "timeout_ms" => {
                        timeout_ms = Some(value.parse().map_err(|_| {
                            ParseError::at(lineno, format!("invalid timeout_ms: {value:?}"))
                        })?)
                    }
                    "shuffle_env" => shuffle_env = parse_bool(value, lineno)?,
                    "seed" => {
                        seed = value.parse().map_err(|_| {
                            ParseError::at(lineno, format!("invalid seed: {value:?}"))
                        })?
                    }
                    "workdir" => workdir = Some(value.to_string()),
                    "expect" => {
                        expect = Expectation::parse(value)
                            .map_err(|e| ParseError::at(lineno, e.message))?
                    }
                    other => {
                        return Err(ParseError::at(
                            lineno,
                            format!("unknown [run] key: {other:?}"),
                        ))
                    }
                },
                "env" => {
                    // Preserve original case for env-var names.
                    let (orig_key, _) = split_kv(line).unwrap();
                    env.insert(orig_key.to_string(), value.to_string());
                }
                "normalize" => match key.as_str() {
                    "strip_timestamps" => normalize.strip_timestamps = parse_bool(value, lineno)?,
                    "strip_hex" => normalize.strip_hex = parse_bool(value, lineno)?,
                    "strip_temp_paths" => normalize.strip_temp_paths = parse_bool(value, lineno)?,
                    "trim_trailing_ws" => normalize.trim_trailing_ws = parse_bool(value, lineno)?,
                    "sort_lines" => normalize.sort_lines = parse_bool(value, lineno)?,
                    "drop_substring" => normalize.drop_substrings.push(value.to_string()),
                    other => {
                        return Err(ParseError::at(
                            lineno,
                            format!("unknown [normalize] key: {other:?}"),
                        ))
                    }
                },
                other => {
                    return Err(ParseError::at(
                        lineno,
                        format!("unknown section: {other:?}"),
                    ))
                }
            }
        }

        if command.is_empty() {
            return Err(ParseError::new(
                "scenario is missing a `command` in the [run] section".to_string(),
            ));
        }
        if runs == 0 {
            return Err(ParseError::new("runs must be >= 1".to_string()));
        }
        if name.is_empty() {
            name = "unnamed-scenario".to_string();
        }

        Ok(Scenario {
            name,
            description,
            command,
            runs,
            timeout_ms,
            shuffle_env,
            seed,
            workdir,
            expect,
            env,
            normalize,
        })
    }
}

/// Remove a trailing `#` or `;` comment that is not inside quotes.
fn strip_comment(line: &str) -> &str {
    let mut in_single = false;
    let mut in_double = false;
    for (i, ch) in line.char_indices() {
        match ch {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            '#' | ';' if !in_single && !in_double => return &line[..i],
            _ => {}
        }
    }
    line
}

fn split_kv(line: &str) -> Option<(&str, &str)> {
    let eq = line.find('=')?;
    let key = line[..eq].trim();
    let value = line[eq + 1..].trim();
    if key.is_empty() {
        return None;
    }
    Some((key, value))
}

fn parse_bool(value: &str, lineno: usize) -> Result<bool, ParseError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "yes" | "1" | "on" => Ok(true),
        "false" | "no" | "0" | "off" => Ok(false),
        other => Err(ParseError::at(
            lineno,
            format!("invalid boolean: {other:?}"),
        )),
    }
}

/// Split a command string into tokens, honoring single and double quotes.
fn tokenize_command(input: &str) -> Result<Vec<String>, ParseError> {
    let mut tokens = Vec::new();
    let mut cur = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut has_token = false;

    for ch in input.chars() {
        match ch {
            '\'' if !in_double => {
                in_single = !in_single;
                has_token = true;
            }
            '"' if !in_single => {
                in_double = !in_double;
                has_token = true;
            }
            c if c.is_whitespace() && !in_single && !in_double => {
                if has_token {
                    tokens.push(std::mem::take(&mut cur));
                    has_token = false;
                }
            }
            c => {
                cur.push(c);
                has_token = true;
            }
        }
    }
    if in_single || in_double {
        return Err(ParseError::new("unterminated quote in command".to_string()));
    }
    if has_token {
        tokens.push(cur);
    }
    Ok(tokens)
}

/// Error produced while parsing a scenario file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub line: Option<usize>,
    pub message: String,
}

impl ParseError {
    fn new(message: String) -> ParseError {
        ParseError {
            line: None,
            message,
        }
    }
    fn at(line: usize, message: String) -> ParseError {
        ParseError {
            line: Some(line),
            message,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(l) => write!(f, "scenario parse error (line {l}): {}", self.message),
            None => write!(f, "scenario parse error: {}", self.message),
        }
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_scenario() {
        let s = Scenario::parse("[run]\ncommand = echo hello").unwrap();
        assert_eq!(s.command, vec!["echo", "hello"]);
        assert_eq!(s.runs, 10);
        assert_eq!(s.expect, Expectation::Unspecified);
    }

    #[test]
    fn parses_full_scenario() {
        let text = r#"
            name = demo
            description = a demo scenario
            [run]
            command = "my prog" --flag 'a b'
            runs = 5
            timeout_ms = 1500
            shuffle_env = true
            seed = 42
            expect = deterministic
            [env]
            GREETING = hi there
            [normalize]
            sort_lines = true
            drop_substring = noise
        "#;
        let s = Scenario::parse(text).unwrap();
        assert_eq!(s.name, "demo");
        assert_eq!(s.command, vec!["my prog", "--flag", "a b"]);
        assert_eq!(s.runs, 5);
        assert_eq!(s.timeout_ms, Some(1500));
        assert!(s.shuffle_env);
        assert_eq!(s.seed, 42);
        assert_eq!(s.expect, Expectation::Deterministic);
        assert_eq!(s.env.get("GREETING").map(String::as_str), Some("hi there"));
        assert!(s.normalize.sort_lines);
        assert_eq!(s.normalize.drop_substrings, vec!["noise".to_string()]);
    }

    #[test]
    fn comments_and_blanks_ignored() {
        let text = "# top comment\n\n[run] ; section\ncommand = echo hi ; trailing\n";
        let s = Scenario::parse(text).unwrap();
        assert_eq!(s.command, vec!["echo", "hi"]);
    }

    #[test]
    fn arg_accumulation() {
        let text = "[run]\ncommand = prog\narg = one\narg = two";
        let s = Scenario::parse(text).unwrap();
        assert_eq!(s.command, vec!["prog", "one", "two"]);
    }

    #[test]
    fn missing_command_is_error() {
        let err = Scenario::parse("name = x").unwrap_err();
        assert!(err.message.contains("command"));
    }

    #[test]
    fn unterminated_quote_errors() {
        let err = Scenario::parse("[run]\ncommand = \"oops").unwrap_err();
        assert!(err.message.contains("unterminated"));
    }

    #[test]
    fn bad_bool_reports_line() {
        let err = Scenario::parse("[run]\ncommand = x\nshuffle_env = maybe").unwrap_err();
        assert_eq!(err.line, Some(3));
    }
}
