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
#[derive(Debug, Clone)]
pub struct OutcomeGroup {
    pub fingerprint: String,
    pub count: u32,
    /// Run indices (0-based) that produced this fingerprint.
    pub run_indices: Vec<u32>,
    /// A representative canonical output for this group.
    pub sample: String,
}

/// Whether the observed behavior satisfied the scenario's expectation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// All runs identical (and this matched the expectation, if any).
    Stable,
    /// Runs diverged (and this matched the expectation, if any).
    Diverged,
    /// Observation contradicted the stated `expect`.
    ExpectationViolated,
}

impl Verdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Stable => "stable",
            Verdict::Diverged => "diverged",
            Verdict::ExpectationViolated => "expectation_violated",
        }
    }
}

/// The full analysis of a scenario.
#[derive(Debug, Clone)]
pub struct Report {
    pub scenario_name: String,
    pub description: String,
    pub command: Vec<String>,
    pub total_runs: u32,
    pub distinct_outcomes: u32,
    pub expectation: Expectation,
    pub verdict: Verdict,
    pub groups: Vec<OutcomeGroup>,
    pub outcomes: Vec<Outcome>,
    pub min_duration_ms: u128,
    pub max_duration_ms: u128,
    pub timeouts: u32,
}

impl Report {
    /// True if the process should exit non-zero (a CI-actionable failure).
    pub fn is_failure(&self) -> bool {
        self.verdict == Verdict::ExpectationViolated
    }

    /// Serialize the report to a [`Json`] value for evidence emission.
    pub fn to_json(&self) -> Json {
        let groups = self
            .groups
            .iter()
            .map(|g| {
                Json::Object(vec![
                    ("fingerprint".into(), Json::str(&g.fingerprint)),
                    ("count".into(), Json::UInt(g.count as u64)),
                    (
                        "run_indices".into(),
                        Json::Array(
                            g.run_indices
                                .iter()
                                .map(|i| Json::UInt(*i as u64))
                                .collect(),
                        ),
                    ),
                    ("sample".into(), Json::str(&g.sample)),
                ])
            })
            .collect();

        let outcomes = self
            .outcomes
            .iter()
            .map(|o| {
                Json::Object(vec![
                    ("run_index".into(), Json::UInt(o.run_index as u64)),
                    ("seed".into(), Json::UInt(o.seed)),
                    ("fingerprint".into(), Json::str(&o.fingerprint)),
                    (
                        "exit_code".into(),
