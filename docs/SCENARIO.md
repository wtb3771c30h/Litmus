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
