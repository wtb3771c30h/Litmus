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
