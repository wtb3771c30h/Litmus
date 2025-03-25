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

function parseGroup(value: unknown, index: number): OutcomeGroup {
  if (!isObject(value)) {
    throw new EvidenceError(`groups[${index}] must be an object`);
  }
  const run_indices = requireArray(value, "run_indices").map((n, i) => {
    if (typeof n !== "number" || !Number.isInteger(n)) {
      throw new EvidenceError(`groups[${index}].run_indices[${i}] must be an integer`);
    }
    return n;
  });
  return {
    fingerprint: requireString(value, "fingerprint"),
    count: requireNumber(value, "count"),
    run_indices,
    sample: requireString(value, "sample"),
  };
}

function parseOutcome(value: unknown, index: number): Outcome {
  if (!isObject(value)) {
    throw new EvidenceError(`outcomes[${index}] must be an object`);
  }
  const rawExit = value["exit_code"];
  let exit_code: number | null;
  if (rawExit === null) {
    exit_code = null;
  } else if (typeof rawExit === "number" && Number.isInteger(rawExit)) {
    exit_code = rawExit;
  } else {
    throw new EvidenceError(`outcomes[${index}].exit_code must be an integer or null`);
  }
  return {
    run_index: requireNumber(value, "run_index"),
    seed: requireNumber(value, "seed"),
    fingerprint: requireString(value, "fingerprint"),
    exit_code,
    timed_out: requireBool(value, "timed_out"),
    duration_ms: requireNumber(value, "duration_ms"),
  };
}

/** Parse and validate a raw JSON string into typed {@link Evidence}. */
export function parseEvidence(raw: string): Evidence {
  let data: unknown;
  try {
    data = JSON.parse(raw);
  } catch (err) {
    throw new EvidenceError(
      `input is not valid JSON: ${(err as Error).message}`,
    );
  }
  return validateEvidence(data);
}

/** Validate an already-parsed value into typed {@link Evidence}. */
export function validateEvidence(data: unknown): Evidence {
  if (!isObject(data)) {
    throw new EvidenceError("evidence must be a JSON object");
  }
  const schema = requireString(data, "schema");
  if (!schema.startsWith("litmus/evidence@")) {
    throw new EvidenceError(`unexpected schema: ${JSON.stringify(schema)}`);
  }

  const groups = requireArray(data, "groups").map(parseGroup);
  const outcomes = requireArray(data, "outcomes").map(parseOutcome);
  const command = requireArray(data, "command").map((c, i) => {
    if (typeof c !== "string") {
      throw new EvidenceError(`command[${i}] must be a string`);
    }
    return c;
  });

  return {
    schema,
    scenario: requireString(data, "scenario"),
    description: requireString(data, "description"),
    command,
    total_runs: requireNumber(data, "total_runs"),
    distinct_outcomes: requireNumber(data, "distinct_outcomes"),
    expectation: asExpectation(requireString(data, "expectation")),
    verdict: asVerdict(requireString(data, "verdict")),
    timeouts: requireNumber(data, "timeouts"),
    min_duration_ms: requireNumber(data, "min_duration_ms"),
    max_duration_ms: requireNumber(data, "max_duration_ms"),
    groups,
    outcomes,
  };
}
