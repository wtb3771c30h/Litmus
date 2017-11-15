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
