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
3. Whatever divergence *survives* both steps is **real** — so litmus fingerprints it, groups
   identical outcomes, and counts how often each occurs.

Collapse to a single fingerprint and the command is stable *at the resolution of your
normalization bench*; two or more fingerprints mean genuine divergence, and the groups tell
you its shape.

## Apparatus

```text
scenario.litmus ─parse─► [ run ×N → normalize → FNV-1a → group ] ─┬─► text report (stdout)
   [run] [env] [normalize]      per-run seed + env + timeout       └─► JSON evidence ─► TS renderer ─► report.html
```

Two instruments share one evidence format:

- **`litmus` (Rust)** — executor and analyzer, standard library only. Parses scenarios, runs
  the command with a per-run timeout, canonicalizes and fingerprints each run, groups
  outcomes, and prints a text report or JSON evidence.
- **`litmus-report` (TypeScript)** — a dependency-free renderer that turns
  `litmus/evidence@1` JSON into a **self-contained HTML** page (inlined CSS, no external
  assets) or plain text, plus a typed *scenario authoring* helper for generating `.litmus`
  files from a matrix.

<p align="center">
  <img src="docs/assets/lab-terminal.svg" alt="litmus laboratory terminal showing a stable report and a diverged report" width="760">
</p>

## First experiments

**Experiment A — a clock that looks flaky but isn't.** The command prints the current time,
so its *raw* output changes every run; after the bench masks timestamps, all 15 runs
fingerprint identically:

```text
$ litmus run scenarios/normalized-timestamp.litmus --sample-lines 2
litmus :: normalized-timestamp
  Raw output varies by wall-clock time, but is stable after normalization.
  command      : node -e process.stdout.write('event at ' + new Date().toISOString() + ' processed\n')
  runs         : 15  (1 distinct outcome)
  expectation  : deterministic
  duration     : 52ms .. 54ms
  verdict      : STABLE

  outcome groups:
  [0] ████████████████████████  15/15  (100.0%)  fp=ba9b8db45a8f6fe6
      runs: 0-14
      | exit=0
      | ... (5 more lines)
```

**Experiment B — genuine flakiness.** A command that randomly emits one of several payloads
and occasionally exits non-zero. Normalization can't erase the divergence because the
randomness *is the content* — so the bench surfaces four distinct outcomes and their
frequencies, including the run that exited `1`:

```text
$ litmus run scenarios/flaky-random-outcome.litmus --runs 12 --sample-lines 1
litmus :: flaky-random-outcome
  Randomly prints one of several results and sometimes fails.
  runs         : 12  (4 distinct outcomes)
  expectation  : flaky
  verdict      : DIVERGED

  outcome groups:
  [0] ████████░░░░░░░░░░░░░░░░   4/12  ( 33.3%)  fp=4688c9756e89ec64   runs: 3, 5, 7-8
  [1] ██████░░░░░░░░░░░░░░░░░░   3/12  ( 25.0%)  fp=2c28612ab0da1d99   runs: 1, 6, 9
  [2] ██████░░░░░░░░░░░░░░░░░░   3/12  ( 25.0%)  fp=9e73afba2fac4352   runs: 2, 4, 11
  [3] ████░░░░░░░░░░░░░░░░░░░░   2/12  ( 16.7%)  fp=fe920309f32cf24f   runs: 0, 10  | exit=1
```

## Scenario grammar

A scenario is a tiny `key = value` dialect with `[section]` headers — INI-like,
hand-authorable, and mirrored by the TypeScript author helper. Full reference:
[`docs/SCENARIO.md`](docs/SCENARIO.md).

```ini
name = greeting
description = a deterministic greeting

[run]
command = node -e "process.stdout.write('hello\n')"   # shell-like quoting
runs = 12
timeout_ms = 5000
expect = deterministic          # deterministic | flaky | unspecified

[env]
GREETING = hello                # names keep their original case

[normalize]
strip_timestamps = true
drop_substring = pid=           # repeatable
```

Grammar notes, all enforced by the parser in [`src/scenario.rs`](src/scenario.rs):

- **Comments** begin with `#` or `;` and run to end of line — even after a value — *unless*
  the marker sits inside single or double quotes.
- **Keys** are case-insensitive (env-var names in `[env]` keep their case). **Booleans**
  accept `true/false`, `yes/no`, `1/0`, `on/off`.
- **`command`** is tokenized honoring single and double quotes, so `"my program" --name 'a b'`
  becomes three tokens. An unterminated quote is a parse error; awkward values can use
  repeated `arg =` lines instead.
- Parse errors report the **1-based line number**, e.g.
  `scenario parse error (line 3): invalid boolean: "maybe"`. Defaults: `runs = 10`,
  `seed = 0`, `expect = unspecified`; normalization passes default **on** except `sort_lines`.

## Seed & environment injection

Reproducibility is opt-in from the target's side. Every run receives two synthetic
environment variables in addition to your `[env]` entries:

- **`LITMUS_SEED`** — the derived per-run seed, `seed + run_index` (wrapping). A program can
  seed its PRNG from this to become reproducible.
- **`LITMUS_RUN`** — the 0-based run index.

Because the derived seed changes per run, a program that *reads* `LITMUS_SEED` varies
deterministically across the suite, while one that pins a constant seed internally is
identical every run — both are legitimate experiments. The
[`seeded-deterministic`](scenarios/seeded-deterministic.litmus) scenario shows the
"pin a constant" flavor.

**Environment ordering** is its own nondeterminism source: some programs are sensitive to
the *insertion order* of variables. Set `shuffle_env = true` and litmus permutes the `[env]`
order with a seeded Fisher–Yates shuffle (`SplitMix64`, driven by `seed ^ 0xa5a5a5a5`), so
the shuffle is reproducible per run while still surfacing order-sensitivity.

