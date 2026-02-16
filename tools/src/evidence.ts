/**
 * Type definitions for litmus JSON evidence (schema `litmus/evidence@1`).
 *
 * These mirror the structure emitted by the Rust `litmus emit` command. The
 * renderer and authoring helper share these types. No runtime dependencies.
 */

export type Verdict = "stable" | "diverged" | "expectation_violated";

export type Expectation = "deterministic" | "flaky" | "unspecified";

export interface OutcomeGroup {
  readonly fingerprint: string;
  readonly count: number;
  readonly run_indices: readonly number[];
  readonly sample: string;
}

export interface Outcome {
  readonly run_index: number;
  readonly seed: number;
  readonly fingerprint: string;
  readonly exit_code: number | null;
  readonly timed_out: boolean;
  readonly duration_ms: number;
}

export interface Evidence {
  readonly schema: string;
  readonly scenario: string;
  readonly description: string;
  readonly command: readonly string[];
  readonly total_runs: number;
  readonly distinct_outcomes: number;
  readonly expectation: Expectation;
  readonly verdict: Verdict;
  readonly timeouts: number;
  readonly min_duration_ms: number;
  readonly max_duration_ms: number;
  readonly groups: readonly OutcomeGroup[];
  readonly outcomes: readonly Outcome[];
}

/** Thrown when a parsed JSON blob does not conform to the evidence schema. */
export class EvidenceError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "EvidenceError";
  }
}
