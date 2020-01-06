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
                        match o.exit_code {
                            Some(c) => Json::Int(c as i64),
                            None => Json::Null,
                        },
                    ),
                    ("timed_out".into(), Json::Bool(o.timed_out)),
                    ("duration_ms".into(), Json::UInt(o.duration_ms as u64)),
                ])
            })
            .collect();

        Json::Object(vec![
            ("schema".into(), Json::str("litmus/evidence@1")),
            ("scenario".into(), Json::str(&self.scenario_name)),
            ("description".into(), Json::str(&self.description)),
            (
                "command".into(),
                Json::Array(self.command.iter().map(Json::str).collect()),
            ),
            ("total_runs".into(), Json::UInt(self.total_runs as u64)),
            (
                "distinct_outcomes".into(),
                Json::UInt(self.distinct_outcomes as u64),
            ),
            (
                "expectation".into(),
                Json::str(self.expectation.to_string()),
            ),
            ("verdict".into(), Json::str(self.verdict.as_str())),
            ("timeouts".into(), Json::UInt(self.timeouts as u64)),
            (
                "min_duration_ms".into(),
                Json::UInt(self.min_duration_ms as u64),
            ),
            (
                "max_duration_ms".into(),
                Json::UInt(self.max_duration_ms as u64),
            ),
            ("groups".into(), Json::Array(groups)),
            ("outcomes".into(), Json::Array(outcomes)),
        ])
    }
}

/// Run every repetition of the scenario and produce a [`Report`].
///
/// `seed_env_var`, when set, is injected as an environment variable holding the
/// per-run seed. This lets target programs opt into deterministic behavior
/// (e.g. reading `LITMUS_SEED`) while still exercising the harness.
pub fn investigate(scenario: &Scenario) -> std::io::Result<Report> {
    let mut outcomes: Vec<Outcome> = Vec::with_capacity(scenario.runs as usize);
    let timeout = scenario.timeout_ms.map(Duration::from_millis);

    let env_len = scenario.env.len();

    for run_index in 0..scenario.runs {
        // Each run gets a derived seed so target programs can be reproducible
        // per-run while the suite as a whole varies.
        let seed = scenario.seed.wrapping_add(run_index as u64);

        // Determine env ordering.
        let mut order: Vec<usize> = (0..env_len).collect();
        if scenario.shuffle_env && env_len > 1 {
            let mut rng = Rng::new(seed ^ 0xa5a5_a5a5);
            rng.shuffle(&mut order);
        }
        let mut env = ordered_env(&scenario.env, &order);
        // Expose the per-run seed to the child.
        env.push(("LITMUS_SEED".to_string(), seed.to_string()));
        env.push(("LITMUS_RUN".to_string(), run_index.to_string()));

        let cfg = RunConfig {
            command: &scenario.command,
            env: &env,
            workdir: scenario.workdir.as_deref(),
            timeout,
        };
        let raw = run_once(&cfg)?;

        // Fold exit status + timeout into the fingerprinted material so that a
        // command that only differs by exit code is still flagged as divergent.
        let combined = format!(
            "exit={}\ntimeout={}\n--stdout--\n{}\n--stderr--\n{}",
            raw.exit_code
                .map(|c| c.to_string())
                .unwrap_or_else(|| "none".into()),
            raw.timed_out,
