//! Orchestration and divergence analysis.
//!
//! Given a parsed [`Scenario`], the [`investigate`] function performs all runs,
//! normalizes and fingerprints each outcome, groups identical outcomes, and
//! builds a [`Report`] describing whether (and how) the command diverged.

use std::collections::BTreeMap;
use std::time::Duration;

use crate::hash::{fingerprint, Rng};
use crate::json::Json;
use crate::normalize::normalize;
use crate::runner::{ordered_env, run_once, RunConfig};
use crate::scenario::{Expectation, Scenario};

/// A single observed outcome (one execution).
#[derive(Debug, Clone)]
pub struct Outcome {
    pub run_index: u32,
    pub seed: u64,
    pub fingerprint: String,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub duration_ms: u128,
    /// The normalized, canonical output that was fingerprinted.
    pub canonical: String,
}

/// A group of runs that shared the same fingerprint.
