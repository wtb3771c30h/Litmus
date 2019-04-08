//! Output normalization.
//!
//! Raw program output is full of legitimately-varying noise: timestamps,
//! heap addresses, temp-file paths, and trailing whitespace. To decide whether
//! two runs are *meaningfully* different we canonicalize the text first, then
//! fingerprint the canonical form. All transformations are pure and
//! hand-written (no regex crate) so we stay stdlib-only.

use crate::scenario::NormalizeOptions;

/// Apply the configured normalization passes to `input`, returning canonical text.
pub fn normalize(input: &str, opts: &NormalizeOptions) -> String {
    let mut lines: Vec<String> = input.lines().map(|l| l.to_string()).collect();

    for line in lines.iter_mut() {
        if opts.strip_hex {
            *line = mask_hex(line);
        }
        if opts.strip_timestamps {
            *line = mask_timestamps(line);
        }
        if opts.strip_temp_paths {
            *line = mask_temp_paths(line);
        }
        if opts.trim_trailing_ws {
            let trimmed = line.trim_end();
            if trimmed.len() != line.len() {
                *line = trimmed.to_string();
            }
        }
    }

    if !opts.drop_substrings.is_empty() {
        lines.retain(|l| !opts.drop_substrings.iter().any(|d| l.contains(d.as_str())));
    }

    if opts.sort_lines {
        lines.sort();
    }

    let mut out = lines.join("\n");
    // Ensure a single trailing newline so "foo" and "foo\n" canonicalize equal.
    out.push('\n');
    out
}

/// Replace hexadecimal runs (typically pointers like `0x7ffee3b1`) with `<hex>`.
///
/// A "hex run" is a `0x` prefix followed by at least three hex digits, or a
/// bare run of six or more hex digits (to catch addresses printed without the
/// prefix) that is not part of a larger alphanumeric word.
fn mask_hex(line: &str) -> String {
    let bytes: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(line.len());
    let mut i = 0;
    while i < bytes.len() {
        // 0x-prefixed form.
        if bytes[i] == '0' && i + 1 < bytes.len() && (bytes[i + 1] == 'x' || bytes[i + 1] == 'X') {
            let mut j = i + 2;
            while j < bytes.len() && bytes[j].is_ascii_hexdigit() {
                j += 1;
            }
            if j - (i + 2) >= 3 {
                out.push_str("<hex>");
                i = j;
                continue;
            }
        }
        // Bare hex run, only when isolated by non-alphanumeric boundaries.
        if bytes[i].is_ascii_hexdigit() {
            let start = i;
            let mut j = i;
            while j < bytes.len() && bytes[j].is_ascii_hexdigit() {
                j += 1;
            }
            let run_len = j - start;
            let left_ok = start == 0 || !is_word_char(bytes[start - 1]);
            let right_ok = j >= bytes.len() || !is_word_char(bytes[j]);
            let has_letter = bytes[start..j].iter().any(|c| c.is_ascii_alphabetic());
            if run_len >= 6 && left_ok && right_ok && has_letter {
                out.push_str("<hex>");
                i = j;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    out
}

fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Replace ISO-8601-ish timestamps and `HH:MM:SS` clocks with `<ts>`.
///
/// Recognizes:
///   * `2026-08-31T21:41:39` (optionally with fractional seconds and zone)
///   * `2026-08-31 21:41:39`
///   * standalone `21:41:39` clocks
fn mask_timestamps(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(line.len());
    let mut i = 0;
    while i < chars.len() {
        if let Some(end) = match_date(&chars, i) {
            out.push_str("<ts>");
            i = end;
            continue;
        }
        if let Some(end) = match_clock(&chars, i) {
            // Only mask a bare clock if not immediately preceded by a digit
            // (which would indicate it is part of something already handled).
            out.push_str("<ts>");
            i = end;
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Match a full date(+time) starting at `i`, returning the end index if matched.
fn match_date(c: &[char], i: usize) -> Option<usize> {
    // YYYY-MM-DD
    if !run_of_digits(c, i, 4) {
        return None;
    }
    let mut j = i + 4;
    if !take(c, &mut j, '-') || !run_of_digits(c, j, 2) {
        return None;
    }
    j += 2;
    if !take(c, &mut j, '-') || !run_of_digits(c, j, 2) {
        return None;
    }
    j += 2;
    // Optional time separated by 'T' or a single space.
    let mut k = j;
    if k < c.len() && (c[k] == 'T' || c[k] == ' ') {
        let sep = k;
        k += 1;
        if let Some(end) = match_clock(c, k) {
            k = end;
            // fractional seconds
            if k < c.len() && c[k] == '.' {
                let mut f = k + 1;
                while f < c.len() && c[f].is_ascii_digit() {
                    f += 1;
                }
                if f > k + 1 {
                    k = f;
                }
            }
            // zone: Z or +HH:MM / -HH:MM
            if k < c.len() && c[k] == 'Z' {
                k += 1;
            } else if k < c.len() && (c[k] == '+' || c[k] == '-') {
                let mut z = k + 1;
                if run_of_digits(c, z, 2) {
                    z += 2;
                    if z < c.len() && c[z] == ':' {
                        z += 1;
                    }
                    if run_of_digits(c, z, 2) {
                        z += 2;
                        k = z;
                    }
                }
            }
            return Some(k);
        }
        // 'T'/space was not actually followed by a clock: only consume the date.
        let _ = sep;
    }
    Some(j)
}

/// Match `HH:MM:SS` (optionally `HH:MM:SS`) starting at `i`.
fn match_clock(c: &[char], i: usize) -> Option<usize> {
    if !run_of_digits(c, i, 2) {
        return None;
    }
