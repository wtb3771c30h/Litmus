/**
 * Tests for the litmus TypeScript tooling, using the built-in `node:test`
 * runner and `node:assert` — no third-party test framework.
 */

import { test } from "node:test";
import assert from "node:assert/strict";

import { parseEvidence, validateEvidence } from "./parse.js";
import { EvidenceError } from "./evidence.js";
import { escapeHtml, renderHtml, renderText, summarizeIndices } from "./render.js";
import { quoteToken, toScenarioText } from "./author.js";

const VALID = JSON.stringify({
  schema: "litmus/evidence@1",
  scenario: "demo",
  description: "a demo",
  command: ["echo", "hi"],
  total_runs: 4,
  distinct_outcomes: 2,
  expectation: "flaky",
