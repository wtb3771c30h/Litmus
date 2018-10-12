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
