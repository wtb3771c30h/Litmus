/**
 * A dependency-free helper for *authoring* litmus scenario files
 * programmatically. This mirrors the Rust parser's grammar so scenarios can be
 * generated (e.g. from a test matrix) and written out as `.litmus` files.
 */

export type Expectation = "deterministic" | "flaky" | "unspecified";

export interface NormalizeSpec {
  strip_timestamps?: boolean;
  strip_hex?: boolean;
  strip_temp_paths?: boolean;
  trim_trailing_ws?: boolean;
  sort_lines?: boolean;
  drop_substrings?: readonly string[];
}

export interface ScenarioSpec {
  name: string;
