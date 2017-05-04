/**
 * Tests for the litmus TypeScript tooling, using the built-in `node:test`
 * runner and `node:assert` — no third-party test framework.
 */

import { test } from "node:test";
import assert from "node:assert/strict";

import { parseEvidence, validateEvidence } from "./parse.js";
import { EvidenceError } from "./evidence.js";
