/**
 * litmus-report — a dependency-free CLI that turns litmus JSON evidence into an
 * HTML (or text) report, and can also author scenario files.
 *
 * Usage:
 *   node dist/cli.js html <evidence.json> [--out report.html]
 *   node dist/cli.js text <evidence.json>
 *   node dist/cli.js --help
 *
 * Reads from stdin when the evidence path is "-".
 */

import { readFileSync, writeFileSync } from "node:fs";
import { parseEvidence } from "./parse.js";
import { renderHtml, renderText } from "./render.js";

