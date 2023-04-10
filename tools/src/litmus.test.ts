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
  verdict: "diverged",
  timeouts: 0,
  min_duration_ms: 1,
  max_duration_ms: 3,
  groups: [
    {
      fingerprint: "aa",
      count: 3,
      run_indices: [0, 1, 2],
      sample: "out a",
    },
    {
      fingerprint: "bb",
      count: 1,
      run_indices: [3],
      sample: "out b",
    },
  ],
  outcomes: [
    {
      run_index: 0,
      seed: 0,
      fingerprint: "aa",
      exit_code: 0,
      timed_out: false,
      duration_ms: 1,
    },
  ],
});

test("parseEvidence accepts a valid document", () => {
  const ev = parseEvidence(VALID);
  assert.equal(ev.scenario, "demo");
  assert.equal(ev.groups.length, 2);
  assert.equal(ev.verdict, "diverged");
});

test("parseEvidence rejects invalid JSON", () => {
  assert.throws(() => parseEvidence("{not json"), EvidenceError);
});

test("validateEvidence rejects wrong schema", () => {
  assert.throws(
    () => validateEvidence({ ...JSON.parse(VALID), schema: "other@1" }),
    /unexpected schema/,
  );
});

test("validateEvidence rejects bad verdict", () => {
  assert.throws(
    () => validateEvidence({ ...JSON.parse(VALID), verdict: "boom" }),
    /unknown verdict/,
  );
});

test("validateEvidence rejects non-integer run_indices", () => {
  const data = JSON.parse(VALID);
  data.groups[0].run_indices = [0, 1.5];
  assert.throws(() => validateEvidence(data), /run_indices/);
});

test("validateEvidence allows null exit_code", () => {
  const data = JSON.parse(VALID);
  data.outcomes[0].exit_code = null;
  const ev = validateEvidence(data);
  assert.equal(ev.outcomes[0].exit_code, null);
});

test("escapeHtml escapes dangerous characters", () => {
  assert.equal(
    escapeHtml(`<script>"'&`),
    "&lt;script&gt;&quot;&#39;&amp;",
  );
});

test("summarizeIndices collapses ranges", () => {
  assert.equal(summarizeIndices([0, 1, 2, 3]), "0-3");
  assert.equal(summarizeIndices([0, 2, 3, 5]), "0, 2-3, 5");
  assert.equal(summarizeIndices([]), "(none)");
  assert.equal(summarizeIndices([5, 4, 3]), "3-5");
});

test("renderHtml produces a self-contained document", () => {
  const html = renderHtml(parseEvidence(VALID));
  assert.match(html, /<!DOCTYPE html>/);
  assert.match(html, /litmus :: demo/);
  assert.match(html, /DIVERGED/);
  assert.match(html, /<style>/);
  // No external asset references.
  assert.doesNotMatch(html, /https?:\/\//);
});

test("renderHtml escapes scenario content", () => {
  const data = JSON.parse(VALID);
  data.scenario = "<evil>";
  const html = renderHtml(validateEvidence(data));
  assert.match(html, /&lt;evil&gt;/);
  assert.doesNotMatch(html, /<evil>/);
});

test("renderText summarizes groups", () => {
  const text = renderText(parseEvidence(VALID));
  assert.match(text, /litmus :: demo/);
  assert.match(text, /\[#0\] aa 3\/4/);
});

test("quoteToken quotes only when needed", () => {
