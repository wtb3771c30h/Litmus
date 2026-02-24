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
    .fp { color: #8ab4f8; }
    .count { margin-left: auto; color: #9aa4b2; }
    .bar { height: 8px; background: #262b36; border-radius: 999px; overflow: hidden; margin: .5rem 0; }
    .bar-fill { height: 100%; background: linear-gradient(90deg,#4f8cff,#8ab4f8); }
    .runs { font-size: .8rem; color: #9aa4b2; margin-bottom: .5rem; }
    .sample { background: #0b0d11; border-radius: 8px; padding: .75rem;
      overflow-x: auto; font-size: .8rem; margin: 0; white-space: pre-wrap; }
    footer { max-width: 960px; margin: 1rem auto 0; color: #6b7280; font-size: .8rem; text-align: center; }
`;

/** Produce a complete HTML document string for the given evidence. */
export function renderHtml(ev: Evidence): string {
  const groupsHtml = ev.groups
    .map((g, i) => renderGroup(g, i, ev.total_runs))
    .join("");

  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>litmus report — ${escapeHtml(ev.scenario)}</title>
  <style>${STYLE}</style>
</head>
<body>
  <main class="card">
    <h1>litmus :: ${escapeHtml(ev.scenario)}</h1>
    <p class="desc">${escapeHtml(ev.description)}</p>
    <p><span class="verdict ${verdictClass(ev.verdict)}">${verdictLabel(ev.verdict)}</span></p>
    <dl class="meta">
      <dt>command</dt><dd><code>${escapeHtml(ev.command.join(" "))}</code></dd>
      <dt>runs</dt><dd>${ev.total_runs} (${ev.distinct_outcomes} distinct)</dd>
      <dt>expectation</dt><dd>${escapeHtml(ev.expectation)}</dd>
      <dt>duration</dt><dd>${ev.min_duration_ms}ms .. ${ev.max_duration_ms}ms</dd>
      <dt>timeouts</dt><dd>${ev.timeouts}</dd>
    </dl>
    <h2 style="font-size:1rem;">Outcome groups</h2>
    ${groupsHtml}
  </main>
  <footer>Generated by the litmus TypeScript renderer · schema ${escapeHtml(ev.schema)}</footer>
</body>
</html>
`;
}

/** Produce a compact plain-text summary (used for terminal previews / tests). */
export function renderText(ev: Evidence): string {
  const lines: string[] = [];
  lines.push(`litmus :: ${ev.scenario}`);
  if (ev.description) lines.push(`  ${ev.description}`);
  lines.push(`  command : ${ev.command.join(" ")}`);
  lines.push(`  runs    : ${ev.total_runs} (${ev.distinct_outcomes} distinct)`);
  lines.push(`  verdict : ${verdictLabel(ev.verdict)}`);
  for (let i = 0; i < ev.groups.length; i++) {
    const g = ev.groups[i];
    lines.push(
      `  [#${i}] ${g.fingerprint} ${g.count}/${ev.total_runs} ` +
        `(${pct(g.count, ev.total_runs).toFixed(1)}%) runs ${summarizeIndices(g.run_indices)}`,
    );
  }
  return lines.join("\n") + "\n";
}
