//! A tiny, dependency-free JSON *writer*.
//!
//! We only ever *emit* JSON evidence (the TypeScript side reads it), so a full
//! parser is unnecessary. This module produces compact, well-formed,
//! spec-compliant JSON with correct string escaping.

use std::fmt::Write as _;

/// A JSON value that can be serialized.
#[derive(Debug, Clone)]
pub enum Json {
    Null,
    Bool(bool),
    /// Integer number.
    Int(i64),
    /// Unsigned number (used for durations / counts).
    UInt(u64),
    Str(String),
    Array(Vec<Json>),
    /// Object with insertion-ordered keys for stable, diff-friendly output.
    Object(Vec<(String, Json)>),
}

impl Json {
    pub fn str(s: impl Into<String>) -> Json {
        Json::Str(s.into())
    }

    /// Serialize to a pretty-printed string with 2-space indentation.
    pub fn to_pretty(&self) -> String {
        let mut out = String::new();
        self.write(&mut out, 0, true);
        out.push('\n');
        out
    }

    /// Serialize to a compact single-line string.
    pub fn to_compact(&self) -> String {
        let mut out = String::new();
        self.write(&mut out, 0, false);
        out
    }

    fn write(&self, out: &mut String, indent: usize, pretty: bool) {
        match self {
