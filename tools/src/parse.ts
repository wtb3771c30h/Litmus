/**
 * Runtime validation of litmus evidence. `JSON.parse` gives us `unknown`;
 * these guards turn it into a strongly-typed {@link Evidence} object or throw
 * an {@link EvidenceError} describing exactly what was wrong. No dependencies.
 */

import {
  Evidence,
  EvidenceError,
  Expectation,
  Outcome,
  OutcomeGroup,
  Verdict,
} from "./evidence.js";

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function requireString(obj: Record<string, unknown>, key: string): string {
  const v = obj[key];
  if (typeof v !== "string") {
    throw new EvidenceError(`field "${key}" must be a string`);
  }
  return v;
}

function requireNumber(obj: Record<string, unknown>, key: string): number {
  const v = obj[key];
  if (typeof v !== "number" || !Number.isFinite(v)) {
    throw new EvidenceError(`field "${key}" must be a finite number`);
  }
  return v;
}

function requireBool(obj: Record<string, unknown>, key: string): boolean {
  const v = obj[key];
  if (typeof v !== "boolean") {
    throw new EvidenceError(`field "${key}" must be a boolean`);
  }
  return v;
}

function requireArray(obj: Record<string, unknown>, key: string): unknown[] {
  const v = obj[key];
  if (!Array.isArray(v)) {
    throw new EvidenceError(`field "${key}" must be an array`);
  }
  return v;
}

const VERDICTS: readonly Verdict[] = [
  "stable",
  "diverged",
  "expectation_violated",
];

const EXPECTATIONS: readonly Expectation[] = [
  "deterministic",
  "flaky",
  "unspecified",
];

function asVerdict(value: string): Verdict {
  if ((VERDICTS as readonly string[]).includes(value)) {
    return value as Verdict;
  }
  throw new EvidenceError(`unknown verdict: ${JSON.stringify(value)}`);
}

function asExpectation(value: string): Expectation {
  if ((EXPECTATIONS as readonly string[]).includes(value)) {
    return value as Expectation;
  }
  throw new EvidenceError(`unknown expectation: ${JSON.stringify(value)}`);
}
