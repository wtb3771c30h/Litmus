//! End-to-end integration tests that exercise the public library API against
//! real child processes. These use the host shell so they run on both Windows
//! and Unix CI.

use litmus::report::investigate;
use litmus::{render_text, Scenario, Verdict};
