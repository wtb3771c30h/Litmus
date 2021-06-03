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
    parts.push(start === prev ? `${start}` : `${start}-${prev}`);
    start = v;
    prev = v;
  }
  parts.push(start === prev ? `${start}` : `${start}-${prev}`);
  return parts.join(", ");
}

function pct(count: number, total: number): number {
  return total > 0 ? (count / total) * 100 : 0;
}

function renderGroup(g: OutcomeGroup, index: number, total: number): string {
  const percent = pct(g.count, total).toFixed(1);
  const width = Math.round(pct(g.count, total));
  return `
      <div class="group">
        <div class="group-head">
          <span class="badge">#${index}</span>
          <code class="fp">${escapeHtml(g.fingerprint)}</code>
          <span class="count">${g.count}/${total} (${percent}%)</span>
        </div>
        <div class="bar"><div class="bar-fill" style="width:${width}%"></div></div>
        <div class="runs">runs: ${escapeHtml(summarizeIndices(g.run_indices))}</div>
        <pre class="sample">${escapeHtml(g.sample)}</pre>
      </div>`;
}

const STYLE = `
    :root { color-scheme: light dark; }
    * { box-sizing: border-box; }
    body {
      font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
      margin: 0; padding: 2rem; line-height: 1.5;
      background: #0f1115; color: #e6e6e6;
    }
    .card {
      max-width: 960px; margin: 0 auto; background: #171a21;
      border: 1px solid #262b36; border-radius: 12px; padding: 1.5rem 2rem;
    }
    h1 { font-size: 1.4rem; margin: 0 0 .25rem; }
    .desc { color: #9aa4b2; margin: 0 0 1rem; }
    .meta { display: grid; grid-template-columns: max-content 1fr; gap: .25rem 1rem;
      font-size: .9rem; margin-bottom: 1rem; }
    .meta dt { color: #9aa4b2; }
    .meta dd { margin: 0; }
    .verdict { display: inline-block; padding: .3rem .8rem; border-radius: 999px;
      font-weight: 700; letter-spacing: .03em; }
    .verdict-stable { background: #123d1f; color: #6ee7a0; }
    .verdict-diverged { background: #3d3312; color: #f5d67a; }
    .verdict-violated { background: #3d1212; color: #f58a8a; }
    .group { border: 1px solid #262b36; border-radius: 10px; padding: 1rem;
      margin: .75rem 0; background: #12151b; }
    .group-head { display: flex; align-items: center; gap: .75rem; flex-wrap: wrap; }
    .badge { background: #262b36; border-radius: 6px; padding: .1rem .5rem; font-size: .8rem; }
