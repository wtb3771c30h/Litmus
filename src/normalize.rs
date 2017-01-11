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
