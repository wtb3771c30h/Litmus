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
