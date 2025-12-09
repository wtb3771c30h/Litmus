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
  // Contains both quote types: fall back to double-quotes and rely on the
  // authoring caller to avoid embedding raw double quotes (documented).
  return `"${token.replace(/"/g, "")}"`;
}

function assertValid(spec: ScenarioSpec): void {
  if (!spec.name || spec.name.trim() === "") {
    throw new Error("scenario name is required");
  }
  if (!spec.command || spec.command.length === 0) {
    throw new Error("scenario command must have at least one token");
  }
  if (spec.runs !== undefined && (!Number.isInteger(spec.runs) || spec.runs < 1)) {
    throw new Error("runs must be an integer >= 1");
  }
  if (
    spec.timeout_ms !== undefined &&
    (!Number.isInteger(spec.timeout_ms) || spec.timeout_ms < 0)
  ) {
    throw new Error("timeout_ms must be a non-negative integer");
  }
}

/** Serialize a {@link ScenarioSpec} into `.litmus` file text. */
export function toScenarioText(spec: ScenarioSpec): string {
  assertValid(spec);
  const lines: string[] = [];

  lines.push(`name = ${spec.name}`);
  if (spec.description) {
    lines.push(`description = ${spec.description}`);
  }
  lines.push("");
  lines.push("[run]");
  lines.push(`command = ${spec.command.map(quoteToken).join(" ")}`);
  if (spec.runs !== undefined) lines.push(`runs = ${spec.runs}`);
  if (spec.timeout_ms !== undefined) lines.push(`timeout_ms = ${spec.timeout_ms}`);
  if (spec.seed !== undefined) lines.push(`seed = ${spec.seed}`);
  if (spec.shuffle_env !== undefined) {
    lines.push(`shuffle_env = ${spec.shuffle_env}`);
  }
  if (spec.workdir !== undefined) lines.push(`workdir = ${spec.workdir}`);
  if (spec.expect !== undefined) lines.push(`expect = ${spec.expect}`);

  if (spec.env && Object.keys(spec.env).length > 0) {
    lines.push("");
    lines.push("[env]");
    for (const [k, v] of Object.entries(spec.env)) {
      lines.push(`${k} = ${v}`);
    }
  }

  const n = spec.normalize;
  if (n && Object.keys(n).length > 0) {
    lines.push("");
    lines.push("[normalize]");
    if (n.strip_timestamps !== undefined)
      lines.push(`strip_timestamps = ${n.strip_timestamps}`);
    if (n.strip_hex !== undefined) lines.push(`strip_hex = ${n.strip_hex}`);
    if (n.strip_temp_paths !== undefined)
      lines.push(`strip_temp_paths = ${n.strip_temp_paths}`);
    if (n.trim_trailing_ws !== undefined)
      lines.push(`trim_trailing_ws = ${n.trim_trailing_ws}`);
    if (n.sort_lines !== undefined) lines.push(`sort_lines = ${n.sort_lines}`);
    if (n.drop_substrings) {
      for (const d of n.drop_substrings) {
        lines.push(`drop_substring = ${d}`);
      }
    }
  }

  return lines.join("\n") + "\n";
}
