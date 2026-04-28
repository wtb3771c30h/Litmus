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
    let mut j = i + 2;
    if !take(c, &mut j, ':') || !run_of_digits(c, j, 2) {
        return None;
    }
    j += 2;
    if !take(c, &mut j, ':') || !run_of_digits(c, j, 2) {
        return None;
    }
    j += 2;
    // Guard against matching a longer digit word like "12:34:56789abc".
    Some(j)
}

fn run_of_digits(c: &[char], start: usize, n: usize) -> bool {
    if start + n > c.len() {
        return false;
    }
    c[start..start + n].iter().all(|ch| ch.is_ascii_digit())
}

fn take(c: &[char], j: &mut usize, ch: char) -> bool {
    if *j < c.len() && c[*j] == ch {
        *j += 1;
        true
    } else {
        false
    }
}

/// Replace common temp-directory paths with `<tmp>`.
///
/// Handles Unix (`/tmp/...`, `/var/folders/...`) and Windows
/// (`C:\Users\x\AppData\Local\Temp\...`, `%TEMP%\...`) shapes.
fn mask_temp_paths(line: &str) -> String {
    let mut result = line.to_string();
    // Unix.
    result = mask_path_prefix(&result, "/tmp/");
    result = mask_path_prefix(&result, "/var/folders/");
    // Windows temp: look for "\Temp\" (case-insensitive) and mask through the
    // next whitespace.
    result = mask_windows_temp(&result);
    result
}

/// Replace `prefix` and everything up to the next whitespace with `<tmp>`.
fn mask_path_prefix(line: &str, prefix: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(pos) = rest.find(prefix) {
        out.push_str(&rest[..pos]);
        out.push_str("<tmp>");
        let after = &rest[pos + prefix.len()..];
        let end = after.find(char::is_whitespace).unwrap_or(after.len());
        rest = &after[end..];
    }
    out.push_str(rest);
    out
}

fn mask_windows_temp(line: &str) -> String {
    let lower = line.to_ascii_lowercase();
    let needle = "\\temp\\";
    let mut out = String::with_capacity(line.len());
    let mut search_from = 0;
    let mut copied_to = 0;
    while let Some(rel) = lower[search_from..].find(needle) {
        let hit = search_from + rel;
        // Walk left to the start of the drive/path token.
        let start = token_start(line, hit);
        // Walk right to the end of the path token.
        let end = token_end(line, hit + needle.len());
        out.push_str(&line[copied_to..start]);
        out.push_str("<tmp>");
        copied_to = end;
        search_from = end;
        if search_from >= lower.len() {
            break;
        }
    }
    out.push_str(&line[copied_to..]);
    out
}

fn token_start(line: &str, mut idx: usize) -> usize {
    let bytes = line.as_bytes();
    while idx > 0 {
        let c = bytes[idx - 1] as char;
        if c.is_whitespace() || c == '"' || c == '\'' {
            break;
        }
        idx -= 1;
    }
    idx
}

fn token_end(line: &str, mut idx: usize) -> usize {
    let bytes = line.as_bytes();
    while idx < bytes.len() {
        let c = bytes[idx] as char;
        if c.is_whitespace() || c == '"' || c == '\'' {
            break;
        }
        idx += 1;
    }
    idx
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts() -> NormalizeOptions {
        NormalizeOptions::default()
    }

    #[test]
    fn trailing_ws_and_newline() {
        let out = normalize("foo   \nbar\t", &opts());
        assert_eq!(out, "foo\nbar\n");
    }

    #[test]
    fn masks_0x_hex() {
        let out = normalize("ptr at 0x7ffee3b1c0d0 done", &opts());
        assert_eq!(out, "ptr at <hex> done\n");
    }

    #[test]
    fn keeps_short_hex_like_versions() {
        // "0x12" has only 2 hex digits -> not masked.
        let out = normalize("v0x12", &opts());
        assert_eq!(out, "v0x12\n");
    }

    #[test]
    fn masks_bare_address_with_letters() {
        let out = normalize("addr deadbeef here", &opts());
        assert_eq!(out, "addr <hex> here\n");
    }

    #[test]
    fn does_not_mask_plain_decimal() {
        let out = normalize("count 123456 items", &opts());
        assert_eq!(out, "count 123456 items\n");
    }

    #[test]
    fn masks_iso_timestamp() {
        let out = normalize("[2026-08-31T21:41:39.347+05:30] start", &opts());
        assert_eq!(out, "[<ts>] start\n");
    }

    #[test]
    fn masks_space_datetime() {
        let out = normalize("at 2026-08-31 21:41:39 now", &opts());
        assert_eq!(out, "at <ts> now\n");
    }

    #[test]
    fn masks_bare_clock() {
        let out = normalize("elapsed 00:00:12 total", &opts());
        assert_eq!(out, "elapsed <ts> total\n");
    }

    #[test]
    fn masks_unix_temp() {
        let out = normalize("wrote /tmp/litmus-abc123/data.bin ok", &opts());
        assert_eq!(out, "wrote <tmp> ok\n");
    }

    #[test]
    fn masks_windows_temp() {
        let line = "path C:\\Users\\joe\\AppData\\Local\\Temp\\x9.tmp end";
        let out = normalize(line, &opts());
        assert_eq!(out, "path <tmp> end\n");
    }

    #[test]
    fn sort_and_drop() {
        let mut o = opts();
        o.sort_lines = true;
        o.drop_substrings = vec!["noise".to_string()];
        let out = normalize("c\na\nnoise line\nb", &o);
        assert_eq!(out, "a\nb\nc\n");
    }
}
