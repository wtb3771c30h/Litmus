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
  }

  let out: string | undefined;
  let inputPath: string | undefined;
  for (let i = 0; i < args.length; i++) {
    const a = args[i];
    if (a === "--out") {
      out = args[++i];
      if (out === undefined) {
        process.stderr.write("litmus-report: --out requires a value\n");
        return 2;
      }
    } else if (a.startsWith("-") && a !== "-") {
      process.stderr.write(`litmus-report: unknown option ${JSON.stringify(a)}\n`);
      return 2;
    } else if (inputPath === undefined) {
      inputPath = a;
    } else {
      process.stderr.write(`litmus-report: unexpected argument ${JSON.stringify(a)}\n`);
      return 2;
    }
  }

  if (inputPath === undefined) {
    process.stderr.write("litmus-report: missing evidence file (use '-' for stdin)\n");
    return 2;
  }

  let raw: string;
  try {
    raw = readInput(inputPath);
  } catch (err) {
    process.stderr.write(`litmus-report: cannot read ${inputPath}: ${(err as Error).message}\n`);
    return 1;
  }

  let output: string;
  try {
    const evidence = parseEvidence(raw);
    output = command === "html" ? renderHtml(evidence) : renderText(evidence);
  } catch (err) {
    process.stderr.write(`litmus-report: ${(err as Error).message}\n`);
    return 1;
  }

  if (out !== undefined) {
    writeFileSync(out, output);
    process.stderr.write(`litmus-report: wrote ${out}\n`);
  } else {
    process.stdout.write(output);
  }
  return 0;
}

// Execute only when run directly (not when imported by tests).
const invokedPath = process.argv[1] ?? "";
if (invokedPath.endsWith("cli.js") || invokedPath.endsWith("cli.ts")) {
  process.exitCode = main(process.argv.slice(2));
}
