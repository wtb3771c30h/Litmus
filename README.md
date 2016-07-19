<p align="center">
  <img src="docs/assets/hero-divergence.svg" alt="litmus — many runs diverging into outcome fingerprints" width="880">
</p>

<h1 align="center">litmus</h1>

<p align="center">
  <em>A bench for nondeterminism.</em> Run any command many times under controlled
  conditions, normalize away the legitimate noise, fingerprint what remains, and read off a
  verdict: <strong>stable</strong>, <strong>diverged</strong>, or
  <strong>expectation&nbsp;violated</strong>.
</p>

<p align="center">
  <code>Rust CLI · std-only</code> · <code>TypeScript · dep-free</code> ·
  <code>Linux / macOS / Windows</code> · <code>schema: litmus/evidence@1</code>
</p>

---

> **The question this notebook answers:** *"Is this test actually broken, or is it flaky?"*
> You pin the command onto the bench, litmus runs the experiment, and the bench prints
> exactly how — and how often — the outcomes disagree, with machine-readable evidence you
> can staple into a bug report or a CI artifact.

## Contents

- [Hypothesis](#hypothesis)
- [Apparatus](#apparatus)
- [First experiments](#first-experiments)
- [Scenario grammar](#scenario-grammar)
- [Seed & environment injection](#seed--environment-injection)
- [The normalization bench](#the-normalization-bench)
- [Fingerprints & outcome grouping](#fingerprints--outcome-grouping)
- [Verdicts: stable / diverged / expectation](#verdicts-stable--diverged--expectation)
- [Evidence → HTML pipeline](#evidence--html-pipeline)
- [Scenario gallery](#scenario-gallery)
- [CI recipes](#ci-recipes)
- [Interpretation cautions](#interpretation-cautions)
- [Install & build](#install--build) · [Command reference](#command-reference) · [Project layout](#project-layout)
- [Troubleshooting](#troubleshooting) · [Limitations](#limitations) · [Roadmap](#roadmap) · [License](#license)

## Hypothesis

Flaky tests erode trust in CI, and their root causes are almost always drawn from a small
catalogue of nondeterminism sources:

| Source | Symptom in output | Bench control |
| --- | --- | --- |
| Wall-clock time | timestamps drift every run | `strip_timestamps` masks `<ts>` |
| Heap / pointer addresses | `0x7ffe…` differs per run | `strip_hex` masks `<hex>` |
| Temp-file names | `/tmp/…`, `…\Temp\…` differ | `strip_temp_paths` masks `<tmp>` |
| Unordered iteration (hash randomization) | lines shuffle | `sort_lines` canonicalizes order |
| Random seeds | payload changes | inject a fixed seed via `LITMUS_SEED` |
| Environment ordering | order-sensitive programs wobble | `shuffle_env` surfaces the sensitivity |
| Genuine timing races | payload *and/or* exit code changes | none — this is the signal, not the noise |

The **working hypothesis of litmus** is that these two populations can be separated
mechanically:

1. **Control** the sources you can (seeds, env, order, timeout, workdir).
2. **Normalize** the noise you can recognize (timestamps, addresses, temp paths, trailing
   whitespace, line order, chosen substrings).
