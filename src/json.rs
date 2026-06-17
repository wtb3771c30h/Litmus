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
            Json::Null => out.push_str("null"),
            Json::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Json::Int(n) => {
                let _ = write!(out, "{n}");
            }
            Json::UInt(n) => {
                let _ = write!(out, "{n}");
            }
            Json::Str(s) => write_escaped(out, s),
            Json::Array(items) => {
                if items.is_empty() {
                    out.push_str("[]");
                    return;
                }
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    if pretty {
                        out.push('\n');
                        push_indent(out, indent + 1);
                    }
                    item.write(out, indent + 1, pretty);
                }
                if pretty {
                    out.push('\n');
                    push_indent(out, indent);
                }
                out.push(']');
            }
            Json::Object(fields) => {
                if fields.is_empty() {
                    out.push_str("{}");
                    return;
                }
                out.push('{');
                for (i, (key, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    if pretty {
                        out.push('\n');
                        push_indent(out, indent + 1);
                    }
                    write_escaped(out, key);
                    out.push(':');
                    if pretty {
                        out.push(' ');
                    }
                    value.write(out, indent + 1, pretty);
                }
                if pretty {
                    out.push('\n');
                    push_indent(out, indent);
                }
                out.push('}');
            }
        }
    }
}

fn push_indent(out: &mut String, level: usize) {
    for _ in 0..level {
        out.push_str("  ");
    }
}

/// Write a JSON string literal with proper escaping per RFC 8259.
fn write_escaped(out: &mut String, s: &str) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_object() {
        let j = Json::Object(vec![
            ("a".into(), Json::Int(1)),
            ("b".into(), Json::Bool(true)),
        ]);
        assert_eq!(j.to_compact(), r#"{"a":1,"b":true}"#);
    }

    #[test]
    fn empty_containers() {
        assert_eq!(Json::Array(vec![]).to_compact(), "[]");
        assert_eq!(Json::Object(vec![]).to_compact(), "{}");
    }

    #[test]
    fn string_escaping() {
        let j = Json::str("line1\ntab\tquote\"back\\slash");
        assert_eq!(j.to_compact(), r#""line1\ntab\tquote\"back\\slash""#);
    }

    #[test]
    fn control_char_escaping() {
        let j = Json::str("\u{01}");
        assert_eq!(j.to_compact(), r#""\u0001""#);
    }

    #[test]
    fn pretty_has_indentation_and_newline() {
        let j = Json::Object(vec![("k".into(), Json::str("v"))]);
        let out = j.to_pretty();
        assert!(out.contains("\n  \"k\": \"v\""));
        assert!(out.ends_with("}\n"));
    }

    #[test]
    fn nested_arrays() {
        let j = Json::Array(vec![
            Json::Int(1),
            Json::Array(vec![Json::Int(2), Json::Int(3)]),
        ]);
        assert_eq!(j.to_compact(), "[1,[2,3]]");
    }
}
