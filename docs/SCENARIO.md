# Scenario File Reference (`.litmus`)

A scenario file tells `litmus` **what command to run**, **how many times**,
**under what conditions**, **how to normalize its output**, and **what you
expect** (stability vs. flakiness). The format is a tiny, dependency-free
`key = value` dialect with `[section]` headers — comparable to INI.

## Quick example

```ini
name = my-scenario
description = one sentence describing what this checks

[run]
command = node -e "console.log('hello')"
runs = 20
timeout_ms = 5000
seed = 12345
shuffle_env = true
workdir = .
expect = deterministic

[env]
GREETING = hello
API_MODE = offline

[normalize]
strip_timestamps = true
strip_hex = true
strip_temp_paths = true
trim_trailing_ws = true
sort_lines = false
drop_substring = pid=
```

## Syntax rules

- **Comments** start with `#` or `;` and run to end of line. They are ignored
  even after a value (`command = echo hi   # trailing comment`), *except* when
  the `#`/`;` is inside single or double quotes.
- **Blank lines** are ignored.
- **Sections** are written `[name]` on their own line. Recognized sections:
  the top level (before any header), `[run]`, `[env]`, and `[normalize]`.
- **Keys** are case-insensitive (except environment-variable names in `[env]`,
  which keep their original case). Values are trimmed of surrounding whitespace.
- **Booleans** accept `true/false`, `yes/no`, `1/0`, `on/off`.
- Parse errors report the **1-based line number**.

## Top-level keys

| Key           | Type   | Default              | Meaning                                  |
| ------------- | ------ | -------------------- | ---------------------------------------- |
| `name`        | string | `unnamed-scenario`   | Human-readable scenario name.            |
| `description` | string | *(empty)*            | One-line description shown in reports.   |

## `[run]` section

| Key           | Type    | Default       | Meaning                                                                 |
| ------------- | ------- | ------------- | ----------------------------------------------------------------------- |
| `command`     | string  | **required**  | Program + arguments. Tokenized with shell-like single/double quoting.   |
| `arg`         | string  | —             | Appends a single argument token (repeatable). Useful for tricky values. |
| `runs`        | integer | `10`          | Number of repetitions (must be ≥ 1).                                    |
| `timeout_ms`  | integer | *(none)*      | Per-run wall-clock timeout in milliseconds. On timeout the child is killed and the run is recorded as timed out. |
| `seed`        | integer | `0`           | Base seed. Each run gets `seed + run_index`.                            |
| `shuffle_env` | bool    | `false`       | Randomize the insertion order of `[env]` variables (deterministic per seed). Surfaces order-sensitive programs. |
| `workdir`     | string  | *(inherited)* | Working directory for the child process.                                |
| `expect`      | enum    | `unspecified` | Assertion: `deterministic`, `flaky`, or `unspecified`.                  |

### Command tokenization

`command` is split into tokens on whitespace, honoring single (`'`) and double
(`"`) quotes so arguments may contain spaces:

```ini
command = "my program" --name 'John Doe' --flag
# tokens: ["my program", "--name", "John Doe", "--flag"]
```

An unterminated quote is a parse error. If quoting is awkward, use repeated
`arg =` lines instead:

```ini
command = my-tool
arg = --title
arg = a value with "both" 'quote' kinds
```

### Injected environment variables

Every run receives two synthetic variables in addition to any `[env]` entries:

- `LITMUS_SEED` — the derived per-run seed (`seed + run_index`).
- `LITMUS_RUN`  — the 0-based run index.

Target programs can read these to become reproducible (seed their PRNG) or to
vary behavior for testing the harness.

## `[env]` section

Each `KEY = VALUE` line adds an environment variable for the child process.
Names preserve their original case. Example:

```ini
[env]
RUST_BACKTRACE = 1
TZ = UTC
```

## `[normalize]` section

Normalization canonicalizes output *before* fingerprinting, so that legitimate
noise does not masquerade as divergence.

| Key                | Type | Default | Effect                                                                 |
| ------------------ | ---- | ------- | ---------------------------------------------------------------------- |
| `strip_timestamps` | bool | `true`  | Replace ISO-8601 / `YYYY-MM-DD HH:MM:SS` / bare `HH:MM:SS` with `<ts>`.|
| `strip_hex`        | bool | `true`  | Replace `0x…` (≥3 hex digits) and bare address-like hex runs with `<hex>`. |
| `strip_temp_paths` | bool | `true`  | Replace `/tmp/…`, `/var/folders/…`, and Windows `…\Temp\…` with `<tmp>`. |
| `trim_trailing_ws` | bool | `true`  | Trim trailing whitespace from each line.                               |
| `sort_lines`       | bool | `false` | Sort lines. Removes order-dependence (e.g. unordered set iteration).   |
| `drop_substring`   | string | —     | Drop any line containing this literal substring (repeatable).          |

The exit code and timeout flag are always folded into the fingerprint, so two
runs that differ *only* by exit status are still reported as divergent.

## Verdicts

After all runs, `litmus` groups outcomes by fingerprint and assigns a verdict:

| `expect`        | Observed stable          | Observed divergent        |
| --------------- | ------------------------ | ------------------------- |
| `deterministic` | `stable`                 | `expectation_violated` ✗  |
| `flaky`         | `expectation_violated` ✗ | `diverged`                |
| `unspecified`   | `stable`                 | `diverged`                |

`litmus check` exits non-zero (`1`) when the verdict is `expectation_violated`,
which is what you wire into CI. `litmus run` always exits `0` unless there is a
usage or execution error (exit `2`).

## Emitting evidence

`litmus emit <scenario> --out evidence.json` writes a JSON document with schema
`litmus/evidence@1` containing the verdict, per-group fingerprints and run
indices, and per-run metadata (exit code, timeout, duration). Feed it to the
TypeScript renderer:

```bash
node tools/dist/cli.js html evidence.json --out report.html
```
