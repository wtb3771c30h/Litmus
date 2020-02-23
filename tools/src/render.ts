/**
 * Renders {@link Evidence} into a self-contained, dependency-free HTML report.
 * The output inlines all CSS so the file can be opened directly in a browser
 * or attached to a CI artifact. No network requests, no external assets.
 */

import { Evidence, OutcomeGroup, Verdict } from "./evidence.js";

/** Escape a string for safe embedding in HTML text/attribute context. */
export function escapeHtml(input: string): string {
  return input
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function verdictLabel(v: Verdict): string {
  switch (v) {
    case "stable":
      return "STABLE";
    case "diverged":
      return "DIVERGED";
    case "expectation_violated":
      return "EXPECTATION VIOLATED";
  }
}

function verdictClass(v: Verdict): string {
  switch (v) {
    case "stable":
      return "verdict-stable";
    case "diverged":
      return "verdict-diverged";
    case "expectation_violated":
      return "verdict-violated";
  }
}

/** Collapse a list of run indices into compact ranges, e.g. "0-3, 5, 7-9". */
export function summarizeIndices(indices: readonly number[]): string {
  if (indices.length === 0) {
    return "(none)";
  }
  const sorted = [...indices].sort((a, b) => a - b);
  const parts: string[] = [];
  let start = sorted[0];
  let prev = sorted[0];
  for (let i = 1; i < sorted.length; i++) {
    const v = sorted[i];
    if (v === prev + 1) {
      prev = v;
      continue;
    }
