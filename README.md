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

## The normalization bench

Normalization runs *before* fingerprinting, so legitimate noise never masquerades as
divergence. Every pass is pure and hand-written (no regex crate), keeping the crate
std-only, applied per line, then optional line-level operations:

| Pass | Default | What it masks / does |
| --- | --- | --- |
| `strip_hex` | on | `0x…` with ≥3 hex digits, and bare address-like hex runs (≥6 hex digits, word-isolated, containing a letter) → `<hex>` |
| `strip_timestamps` | on | ISO-8601 `2026-08-31T21:41:39.347+05:30`, `YYYY-MM-DD HH:MM:SS`, and bare `HH:MM:SS` → `<ts>` |
| `strip_temp_paths` | on | `/tmp/…`, `/var/folders/…`, and Windows `…\Temp\…` → `<tmp>` |
| `trim_trailing_ws` | on | trims trailing whitespace per line |
| `drop_substring` | — | drops any line containing a literal substring (repeatable) |
| `sort_lines` | off | sorts lines, removing order-dependence |

Two deliberate design choices worth knowing:

- **A single trailing newline is always appended** to the canonical form, so `foo` and
  `foo\n` compare equal.
- The masks are conservative to avoid false positives: `strip_hex` won't touch `v0x12`
  (two hex digits) or plain `123456` (no letters), but *will* mask a standalone `deadbeef`.

The `<hex>` / `<ts>` / `<tmp>` tokens are visible in the sample output, so it's easy to
confirm the bench masked what you intended.

## Fingerprints & outcome grouping

Each run is reduced to a fingerprint over a *combined* record:

```text
exit=<code|none>
timeout=<bool>
--stdout--
<normalized stdout>
--stderr--
<normalized stderr>
```

That block is normalized, then hashed with **64-bit FNV-1a** rendered as lowercase hex
(`fp=ba9b8db45a8f6fe6`). FNV-1a is chosen over `DefaultHasher` because its output is
**stable across Rust versions** — essential when fingerprints are persisted into JSON and
compared later. Folding in the exit code and timeout flag means a command that differs
*only* by exit status is still flagged divergent. Runs sharing a fingerprint are grouped;
groups sort by descending frequency, ties broken by fingerprint for stable output.

## Verdicts: stable / diverged / expectation

The verdict is a function of what you asserted (`expect`) and what was observed:

| `expect` | observed **stable** (1 group) | observed **divergent** (2+ groups) |
| --- | --- | --- |
| `deterministic` | `stable` ✓ | `expectation_violated` ✗ |
| `flaky` | `expectation_violated` ✗ | `diverged` ✓ |
| `unspecified` | `stable` | `diverged` |

Note the symmetry: declaring `expect = flaky` and then observing perfect stability is
*also* a violation — useful for asserting that a known-flaky reproduction still reproduces.
`litmus check` exits `1` on `expectation_violated`; that is the signal you wire into CI.

## Evidence → HTML pipeline

`litmus emit` (or `run --out FILE`) writes JSON with schema `litmus/evidence@1`:

```json
{
  "schema": "litmus/evidence@1",
  "scenario": "normalized-timestamp",
  "verdict": "stable",
  "total_runs": 15, "distinct_outcomes": 1, "timeouts": 0,
  "min_duration_ms": 52, "max_duration_ms": 54,
  "groups": [
    { "fingerprint": "ba9b8db45a8f6fe6", "count": 15, "run_indices": [0], "sample": "exit=0\ntimeout=false\n--stdout--\nevent at <ts> processed\n\n--stderr--\n" }
  ],
  "outcomes": [
    { "run_index": 0, "seed": 0, "fingerprint": "ba9b8db45a8f6fe6", "exit_code": 0, "timed_out": false, "duration_ms": 53 }
  ]
}
```

A complete sample lives at [`examples/evidence-sample.json`](examples/evidence-sample.json).
Hand it to the dependency-free renderer to produce a self-contained HTML report or plain
text:

```bash
node tools/dist/cli.js html evidence.json --out report.html   # inlined CSS, no external assets
node tools/dist/cli.js text evidence.json                     # or plain text
node tools/dist/cli.js html - < evidence.json                 # read from stdin
```

## Scenario gallery

Every example uses `node -e` so it behaves identically across platforms. Each one isolates a
single lesson:

| Scenario | Demonstrates | Verdict |
| --- | --- | --- |
| [`deterministic-echo`](scenarios/deterministic-echo.litmus) | A genuinely stable command; defaults are idempotent on clean output. | stable |
| [`normalized-timestamp`](scenarios/normalized-timestamp.litmus) | Raw output varies by clock, but timestamp masking makes it stable. | stable |
| [`seeded-deterministic`](scenarios/seeded-deterministic.litmus) | A pseudo-random program pinned to a fixed internal seed. | stable |
| [`set-order-flaky`](scenarios/set-order-flaky.litmus) | Unordered iteration → nondeterministic line order. | diverged |
| [`set-order-fixed`](scenarios/set-order-fixed.litmus) | The fix: `sort_lines` removes order-dependence. | stable |
| [`flaky-random-outcome`](scenarios/flaky-random-outcome.litmus) | Random payload plus an occasional non-zero exit. | diverged |

The `set-order-*` pair is the most instructive: the *code* is identical between them; the
flakiness lived in the assertion, and `sort_lines` moved comparison to a canonical form.

## CI recipes

Use `check` to fail a build when a scenario's expectation is violated. A minimal gate:

```yaml
# .github/workflows/flaky-gate.yml
- run: cargo build --release
- name: Assert stability
  run: |
    bin=./target/release/litmus
    [ -f "$bin.exe" ] && bin="$bin.exe"
    "$bin" check scenarios/deterministic-echo.litmus
    "$bin" check scenarios/normalized-timestamp.litmus
    "$bin" check scenarios/set-order-fixed.litmus
```

To attach evidence to a run, emit JSON and render HTML as an artifact:

```yaml
- run: ./target/release/litmus emit scenarios/flaky-random-outcome.litmus --out evidence.json
- run: node tools/dist/cli.js html evidence.json --out report.html
- uses: actions/upload-artifact@v4
  with: { name: litmus-report, path: "evidence.json\nreport.html" }
```

The shipped [`.github/workflows/ci.yml`](.github/workflows/ci.yml) goes further: Rust suite
on Linux/macOS/Windows, TypeScript on Linux, and an end-to-end job that emits and uploads a
rendered report.

## Interpretation cautions

A verdict is only as trustworthy as the experiment that produced it:

- **"Stable" means stable at your normalization resolution**, not "correct." Over-aggressive
  masking can hide real divergence — read the sample output and confirm the
  `<ts>`/`<hex>`/`<tmp>` tokens landed where you expected.
- **Absence of divergence is not proof of determinism.** A 1-in-10,000 race needs far more
  than `runs = 12`; raise `--runs` for rare flakes. Group frequencies estimate, not
  guarantee, the true distribution.
- **The harness controls what it can, not the world.** It does not sandbox network,
  filesystem, or clock; external state can leak in and show up as divergence you must judge.
- **Timeouts are recorded, not retried** — a timed-out run is a distinct observation
  (`timeout=true`), not an error. **Duration is wall-clock and noisy**, so treat `min`/`max`
  as informational, not a benchmark.

## Install & build

Requirements: a stable Rust toolchain (CLI) and Node.js ≥ 18 (only for the optional
TypeScript tooling). Nothing is downloaded — both sides are dependency-free.

```bash
# Build the CLI
cargo build --release
# binary at target/release/litmus (litmus.exe on Windows)

# Build & test the TypeScript tooling (optional)
cd tools
npx tsc -p tsconfig.json
node --test dist/litmus.test.js
```

Or drive everything through the [`Makefile`](Makefile):

```bash
make all      # build + test Rust and TypeScript
make demo     # run every example scenario
make emit     # emit evidence and render an HTML report
make lint     # cargo fmt --check + clippy -D warnings
```

## Command reference

| Command | Purpose |
| --- | --- |
| `litmus run <scenario>` | Run the scenario and print a text report. |
| `litmus check <scenario>` | Like `run`, but **exit 1** if the expectation is violated. Use in CI. |
| `litmus emit <scenario>` | Run and write JSON evidence (`--out FILE`, or stdout). |
| `litmus explain <scenario>` | Parse the scenario and print its resolved configuration. |
| `litmus version` / `help` | Version / usage. |

Common options: `--runs N`, `--timeout-ms N`, `--seed N`, `--json`, `--out FILE`,
`--sample-lines N`, `--no-color` (honors `NO_COLOR`).

Exit codes: `0` success · `1` expectation violated (`check` only) · `2` usage or execution
error (unreadable scenario, or the program cannot be spawned).

## Project layout

```text
litmus/
├── Cargo.toml
├── src/
│   ├── main.rs        # CLI: argument parsing, subcommands
│   ├── lib.rs         # library root + text report renderer
│   ├── scenario.rs    # .litmus parser
│   ├── runner.rs      # command execution with timeout
│   ├── normalize.rs   # output canonicalization
│   ├── hash.rs        # FNV-1a fingerprint + SplitMix64 RNG
│   ├── json.rs        # dependency-free JSON writer
│   └── report.rs      # orchestration + divergence analysis
├── tests/integration.rs   # end-to-end tests against real processes
├── scenarios/             # example .litmus files (the gallery)
├── tools/src/             # TS: evidence types, parser, renderer, author, CLI, tests
├── docs/SCENARIO.md       # full scenario reference
├── docs/assets/           # README artwork (animated SVG)
├── examples/evidence-sample.json
├── Makefile
└── .github/workflows/ci.yml
```

## Troubleshooting

- **`scenario is missing a command in the [run] section`** — add a `command`, or accumulate
  arguments with `arg =` lines.
- **`unterminated quote in command`** — a `'` or `"` was opened but not closed; close it or
  switch to repeated `arg =` lines.
- **`scenario parse error (line N): …`** — the parser points at the line. Common causes: a
  bad boolean (only `true/false/yes/no/1/0/on/off`) or an unknown key in a section.
- **Everything reports `diverged` unexpectedly** — a needed normalization pass is off, or the
  noise isn't recognized. Inspect the sample lines, then enable the matching pass or add a
  `drop_substring`.
- **A `deterministic` scenario flips to `expectation_violated`** — the command really
  diverged; the group samples show how, the run indices show which repetitions.
- **`cannot read <path>`** — wrong working directory; `litmus` reads the scenario relative to
  where you invoke it.

## Limitations

- **No sandboxing.** litmus controls seeds, env, order, timeout, and workdir — not network,
  filesystem, or clock. Interpret external divergence accordingly.
- **Line-oriented normalization.** Masks operate per line over a fixed set of shapes
  (ISO-ish timestamps, `0x`/bare hex, common temp roots); exotic formats need a
  `drop_substring` or a code change.
- **Fingerprints are equality, not similarity.** Outcomes match only if their canonical forms
  are byte-identical; litmus does not compute a diff or a distance.
- **Sequential runs.** Repetitions execute one after another — no built-in parallel execution.
- **UTF-8 lossy capture.** Output is decoded with `from_utf8_lossy`; raw binary is not
  preserved byte-for-byte.

## Roadmap

- Parallel run execution with a bounded worker pool.
- A `diff` subcommand showing the minimal delta between two outcome groups.
- More normalization passes (UUIDs, durations, PIDs) behind explicit flags.
- A `--baseline evidence.json` mode for `check`, failing only on *new* fingerprints.
- JUnit-style XML output alongside the HTML renderer for CI dashboards.

## License

MIT — see [`LICENSE`](LICENSE).
