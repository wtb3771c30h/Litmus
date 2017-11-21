/**
 * Type definitions for litmus JSON evidence (schema `litmus/evidence@1`).
 *
 * These mirror the structure emitted by the Rust `litmus emit` command. The
 * renderer and authoring helper share these types. No runtime dependencies.
 */

export type Verdict = "stable" | "diverged" | "expectation_violated";

export type Expectation = "deterministic" | "flaky" | "unspecified";

export interface OutcomeGroup {
