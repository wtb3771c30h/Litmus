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
  description?: string;
  /** Command tokens; the first is the program, the rest are arguments. */
  command: readonly string[];
  runs?: number;
  timeout_ms?: number;
  seed?: number;
  shuffle_env?: boolean;
  workdir?: string;
  expect?: Expectation;
  env?: Readonly<Record<string, string>>;
  normalize?: NormalizeSpec;
}

/** Quote a command token if it contains whitespace or quote characters. */
export function quoteToken(token: string): string {
  if (token.length === 0) {
    return '""';
  }
  if (!/[\s"']/.test(token)) {
    return token;
  }
  if (!token.includes('"')) {
    return `"${token}"`;
  }
  if (!token.includes("'")) {
    return `'${token}'`;
  }
