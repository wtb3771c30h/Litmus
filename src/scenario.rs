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
