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
