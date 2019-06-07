/**
 * litmus-report — a dependency-free CLI that turns litmus JSON evidence into an
 * HTML (or text) report, and can also author scenario files.
 *
 * Usage:
 *   node dist/cli.js html <evidence.json> [--out report.html]
 *   node dist/cli.js text <evidence.json>
 *   node dist/cli.js --help
 *
 * Reads from stdin when the evidence path is "-".
 */

import { readFileSync, writeFileSync } from "node:fs";
import { parseEvidence } from "./parse.js";
import { renderHtml, renderText } from "./render.js";

const HELP = `litmus-report — render litmus evidence

USAGE:
  litmus-report html <evidence.json> [--out FILE]
  litmus-report text <evidence.json>
  litmus-report --help

ARGS:
  <evidence.json>   Path to a litmus evidence file, or "-" for stdin.

OPTIONS:
  --out FILE        Write output to FILE instead of stdout (html mode).
`;

function readInput(path: string): string {
  if (path === "-") {
    return readFileSync(0, "utf8");
  }
  return readFileSync(path, "utf8");
}

export function main(argv: readonly string[]): number {
  const args = [...argv];
  const command = args.shift();

  if (!command || command === "--help" || command === "-h") {
    process.stdout.write(HELP);
    return command ? 0 : 1;
  }

  if (command !== "html" && command !== "text") {
    process.stderr.write(`litmus-report: unknown command ${JSON.stringify(command)}\n`);
    return 2;
