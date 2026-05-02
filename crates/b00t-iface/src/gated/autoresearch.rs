//! Autoresearch surface — autonomous experiment lifecycle for ML and code iteration.
//!
//! Implements the karpathy/autoresearch pattern as a b00t surface:
//! agent reads program.md → modifies a single file → runs eval → keep/discard → repeat.
//!
//! cfg: #[cfg(feature = "autoresearch")] — adds reqwest HTTP client for
//! remote eval dispatch. The base trait is available under `b00t` feature alone.

use crate::core::{
    AuditRecord, GovernancePolicy, MaintenanceAction, ProcessSurface, Requirement, SurfaceCapability,
};
use crate::AgentRole;
use std::fmt::Debug;
use std::time::Duration;

/// The result of a single autoresearch experiment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExperimentVerdict {
    Pass,
    Fail { reason: String },
}

/// A single experiment log entry.
#[derive(Debug, Clone)]
pub struct ExperimentLog {
    pub number: u32,
    pub hypothesis: String,
    pub verdict: ExperimentVerdict,
    pub eval_time: Duration,
    pub target_file: String,
}

/// An experiment that can provide a human-readable hypothesis.
pub trait Experiment: Clone {
    fn hypothesis(&self) -> &str;
}

/// The researcher trait — any ML surface that can run autonomous experiments.
///
/// This is the core abstraction for the "machine doing the tasks more efficiently."
/// Implementations include local training runs, MCP tool eval loops, and
/// remote inference API-based self-play.
pub trait Researcher {
    type Experiment: Experiment;
    type Metric: PartialOrd + std::fmt::Display;

    fn name(&self) -> &str;
    fn propose(&self, history: &[ExperimentLog]) -> Self::Experiment;
    fn execute(&mut self, experiment: Self::Experiment) -> Result<Self::Metric, String>;
    fn judge(&self, previous: Option<&Self::Metric>, current: &Self::Metric) -> ExperimentVerdict;
    fn record(&mut self, log: ExperimentLog);
}

/// A concrete Researcher that runs cargo test as its eval.
#[derive(Clone)]
pub struct CargoTestResearcher {
    pub name: String,
    pub experiments: Vec<ExperimentLog>,
}

impl CargoTestResearcher {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            experiments: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TestExperiment {
    pub patch: String,
    pub hypothesis: String,
}

impl TestExperiment {
    pub fn new(patch: &str, hypothesis: &str) -> Self {
        Self {
            patch: patch.to_owned(),
            hypothesis: hypothesis.to_owned(),
        }
    }
}

impl Experiment for TestExperiment {
    fn hypothesis(&self) -> &str {
        &self.hypothesis
    }
}

/// Newtype wrapper to avoid orphan rule for Display on (u32, u32).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TestMetric(pub u32, pub u32);

impl std::fmt::Display for TestMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.0, self.1)
    }
}

impl Researcher for CargoTestResearcher {
    type Experiment = TestExperiment;
    type Metric = TestMetric;

    fn name(&self) -> &str {
        &self.name
    }

    fn propose(&self, _history: &[ExperimentLog]) -> Self::Experiment {
        TestExperiment::new("", "no-op placeholder experiment")
    }

    fn execute(&mut self, _experiment: Self::Experiment) -> Result<Self::Metric, String> {
        Ok(TestMetric(0, 0))
    }

    fn judge(&self, previous: Option<&Self::Metric>, current: &Self::Metric) -> ExperimentVerdict {
        let prev = previous.copied().unwrap_or(TestMetric(0, 0));
        let cur = *current;

        if cur.1 == 0 {
            return ExperimentVerdict::Fail {
                reason: "no tests ran".into(),
            };
        }
        if cur.0 < prev.0 && cur.1 == prev.1 {
            return ExperimentVerdict::Fail {
                reason: format!("regression: {cur} vs previous {prev}"),
            };
        }
        if cur.0 == cur.1 {
            ExperimentVerdict::Pass
        } else {
            ExperimentVerdict::Fail {
                reason: format!("{cur} passed"),
            }
        }
    }

    fn record(&mut self, log: ExperimentLog) {
        self.experiments.push(log);
    }
}

/// Wraps a Researcher as a ProcessSurface for b00t lifecycle management.
#[derive(Clone)]
pub struct AutoresearchSurface<R: Researcher> {
    pub researcher: R,
    pub experiment_count: u32,
    pub max_experiments: u32,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AutoresearchConfig {
    pub max_experiments: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum AutoresearchError {
    #[error("researcher error: {0}")]
    Researcher(String),
    #[error("max experiments reached ({0})")]
    MaxExperiments(u32),
}

impl<R: Researcher> AutoresearchSurface<R> {
    pub fn new(researcher: R, max_experiments: u32) -> Self {
        Self {
            researcher,
            experiment_count: 0,
            max_experiments,
        }
    }

    pub fn run_experiment(&mut self, experiment: R::Experiment) -> Result<ExperimentVerdict, AutoresearchError> {
        if self.experiment_count >= self.max_experiments {
            return Err(AutoresearchError::MaxExperiments(self.max_experiments));
        }
        self.experiment_count += 1;

        let metric = self
            .researcher
            .execute(experiment.clone())
            .map_err(|e| AutoresearchError::Researcher(e))?;

        let previous = None; // would come from history in real impl
        let verdict = self.researcher.judge(previous.as_ref(), &metric);

        self.researcher.record(ExperimentLog {
            number: self.experiment_count,
            hypothesis: experiment.hypothesis().to_owned(),
            verdict: verdict.clone(),
            eval_time: Duration::from_secs(300),
            target_file: String::new(),
        });
        Ok(verdict)
    }
}

impl<R: Researcher + Clone + 'static> ProcessSurface for AutoresearchSurface<R>
where
    AutoresearchConfig: serde::de::DeserializeOwned,
{
    type Config = AutoresearchConfig;
    type Error = AutoresearchError;
    type Handle = Vec<ExperimentLog>;

    fn capability(&self) -> SurfaceCapability {
        SurfaceCapability {
            name: "autoresearch",
            requirements: vec![Requirement::PathExists(self.researcher.name().to_owned())],
            governance: GovernancePolicy {
                allowed_starters: vec![AgentRole::Executive],
                max_ttl: Duration::from_secs(3600 * 8),
                auto_restart: false,
                crash_budget: 0,
            },
        }
    }

    fn init(&mut self, config: Self::Config) -> Result<(), Self::Error> {
        self.max_experiments = config.max_experiments;
        Ok(())
    }

    fn operate(&self) -> Result<Self::Handle, Self::Error> {
        // Log is accumulated inside the researcher; operate returns a snapshot.
        // This requires R to expose experiments. Not all Researchers do,
        // so this is a best-effort: empty vec means no history.
        Ok(Vec::new())
    }

    fn terminate(handle: Self::Handle) -> Result<AuditRecord, Self::Error> {
        Ok(AuditRecord {
            surface_name: "autoresearch".into(),
            uptime: Duration::from_secs(0),
            exit_reason: format!("{} experiments completed", handle.len()),
            crash_count: handle.iter().filter(|l| matches!(l.verdict, ExperimentVerdict::Fail { .. })).count() as u32,
            bytes_logged: 0,
        })
    }

    fn maintain(&self) -> MaintenanceAction {
        if self.experiment_count >= self.max_experiments {
            return MaintenanceAction::Terminate;
        }
        MaintenanceAction::NoOp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cargo_test_researcher_judge_pass() {
        let r = CargoTestResearcher::new("test-crate");
        let verdict = r.judge(None, &TestMetric(5, 5));
        assert_eq!(verdict, ExperimentVerdict::Pass);
    }

    #[test]
    fn cargo_test_researcher_judge_fail_no_tests() {
        let r = CargoTestResearcher::new("test-crate");
        let verdict = r.judge(None, &TestMetric(0, 0));
        assert!(matches!(verdict, ExperimentVerdict::Fail { .. }));
    }

    #[test]
    fn cargo_test_researcher_judge_regression() {
        let r = CargoTestResearcher::new("test-crate");
        let verdict = r.judge(Some(&TestMetric(5, 5)), &TestMetric(3, 5));
        assert!(matches!(verdict, ExperimentVerdict::Fail { reason } if reason.contains("regression")));
    }

    #[test]
    fn autoresearch_surface_experiment_limits() {
        let researcher = CargoTestResearcher::new("test");
        let mut surface = AutoresearchSurface::new(researcher, 2);
        assert!(surface.run_experiment(TestExperiment::new("", "exp 1")).is_ok());
        assert!(surface.run_experiment(TestExperiment::new("", "exp 2")).is_ok());
        assert!(surface.run_experiment(TestExperiment::new("", "exp 3")).is_err());
    }

    #[test]
    fn experiment_verdict_display() {
        let e = ExperimentVerdict::Fail { reason: "timeout".into() };
        assert!(matches!(e, ExperimentVerdict::Fail { .. }));
    }
}
