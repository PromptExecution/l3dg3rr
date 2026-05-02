//! Ralph loop — typed lifecycle for the propose→execute→judge→record→repeat cycle.
//!
//! A RALPH (Research, Analyze, Learn, Propose, Hone) loop generalizes the
//! autoreearch pattern into a first-class b00t surface. Each loop cycle
//! is a typed transition through the `ProcessSurface` lifecycle with
//! governance-enforced TTL, crash budget, and cadence.
//!
//! The loop variant refers to the idea that any iterative process can be
//! expressed as a Surface with typed properties (TTL, cadence, max_iters)
//! that harmonize with existing `GovernancePolicy`, `SurfaceMachine`, and
//! `SurfaceHarness` infrastructure.
//!
//! # Loop lifecycle
//!
//! ```text
//! Init → [Propose → Execute → Judge → Record → Maintain]^n → Terminate
//! ```
//!
//! Each iteration is a `Maintain` cycle. The loop is governed by:
//! - `max_iterations`: total budget for the loop
//! - `iteration_ttl`: max wall-clock per iteration
//! - `crash_budget`: max failures before forced termination
//! - `cadence`: min delay between iterations

use crate::core::{
    AuditRecord, GovernancePolicy, MaintenanceAction, ProcessSurface, Requirement, SurfaceCapability,
};
use crate::AgentRole;
use crate::gated::autoresearch::{Experiment, ExperimentLog, ExperimentVerdict, Researcher};
use std::time::{Duration, Instant};

/// Properties that define a ralph loop's behavior.
#[derive(Debug, Clone)]
pub struct RalphLoopProperties {
    /// Max total iterations before auto-terminate.
    pub max_iterations: u32,
    /// Max wall-clock time per single iteration.
    pub iteration_ttl: Duration,
    /// Max failures before quarantine.
    pub crash_budget: u32,
    /// Min delay between iterations (cadence / cooldown).
    pub cadence: Duration,
    /// Surface name for governance and audit.
    pub name: String,
}

impl Default for RalphLoopProperties {
    fn default() -> Self {
        Self {
            max_iterations: 10,
            iteration_ttl: Duration::from_secs(300),
            crash_budget: 3,
            cadence: Duration::from_secs(1),
            name: "ralph-loop".into(),
        }
    }
}

/// State of a single ralph loop iteration.
#[derive(Debug, Clone)]
pub struct IterationState {
    pub number: u32,
    pub hypothesis: String,
    pub verdict: ExperimentVerdict,
    pub elapsed: Duration,
    pub crashed: bool,
}

/// A surface that runs an iterative propose→execute→judge→record loop.
///
/// This generalizes the `AutoresearchSurface` pattern by making the loop
/// itself a `ProcessSurface` with typed properties. The inner `Researcher`
/// provides the domain-specific propose/execute/judge/record logic.
#[derive(Clone)]
pub struct RalphLoopSurface<R: Researcher> {
    pub researcher: R,
    pub properties: RalphLoopProperties,
    pub history: Vec<IterationState>,
    pub started_at: Option<Instant>,
    crash_count: u32,
}

impl<R: Researcher> RalphLoopSurface<R> {
    pub fn new(researcher: R, properties: RalphLoopProperties) -> Self {
        Self {
            researcher,
            properties,
            history: Vec::new(),
            started_at: None,
            crash_count: 0,
        }
    }

    /// Run a single loop iteration: propose → execute → judge → record.
    pub fn iterate(&mut self) -> Result<ExperimentVerdict, String> {
        if self.history.len() >= self.properties.max_iterations as usize {
            return Err(format!(
                "max iterations reached: {}",
                self.properties.max_iterations
            ));
        }

        let iter_start = Instant::now();
        let experiment = self.researcher.propose(&self.history.iter().map(|h| ExperimentLog {
            number: h.number,
            hypothesis: h.hypothesis.clone(),
            verdict: h.verdict.clone(),
            eval_time: h.elapsed,
            target_file: String::new(),
        }).collect::<Vec<_>>());

        let hypothesis = experiment.hypothesis().to_owned();
        let number = self.history.len() as u32 + 1;

        // Execute with TTL guard (we just record elapsed)
        let metric = match self.researcher.execute(experiment) {
            Ok(m) => m,
            Err(e) => {
                self.crash_count += 1;
                let elapsed = iter_start.elapsed();
                let iter = IterationState {
                    number,
                    hypothesis,
                    verdict: ExperimentVerdict::Fail {
                        reason: format!("execute error: {e}"),
                    },
                    elapsed,
                    crashed: true,
                };
                self.history.push(iter.clone());
                return Ok(iter.verdict);
            }
        };

        let previous = self.history.last().map(|h| h.number);
        let _ = previous;
        let verdict = self.researcher.judge(None, &metric);
        let elapsed = iter_start.elapsed();

        self.researcher.record(ExperimentLog {
            number,
            hypothesis: hypothesis.clone(),
            verdict: verdict.clone(),
            eval_time: elapsed,
            target_file: String::new(),
        });

        self.history.push(IterationState {
            number,
            hypothesis,
            verdict: verdict.clone(),
            elapsed,
            crashed: false,
        });

        Ok(verdict)
    }

    /// Check if any termination condition is met.
    fn should_terminate(&self) -> bool {
        if self.history.len() >= self.properties.max_iterations as usize {
            return true;
        }
        if self.crash_count >= self.properties.crash_budget {
            return true;
        }
        false
    }

    fn governance(&self) -> GovernancePolicy {
        GovernancePolicy {
            allowed_starters: vec![AgentRole::Executive, AgentRole::Operator],
            max_ttl: self
                .properties
                .iteration_ttl
                .saturating_mul(self.properties.max_iterations as u32),
            auto_restart: false,
            crash_budget: self.properties.crash_budget,
        }
    }
}

impl<R: Researcher + Clone + 'static> ProcessSurface for RalphLoopSurface<R>
where
    RalphLoopConfig: serde::de::DeserializeOwned,
{
    type Config = RalphLoopConfig;
    type Error = RalphLoopError;
    type Handle = Vec<IterationState>;

    fn capability(&self) -> SurfaceCapability {
        SurfaceCapability {
            name: "ralph-loop",
            requirements: vec![Requirement::BinaryOnPath("cargo".into())],
            governance: self.governance(),
        }
    }

    fn init(&mut self, config: Self::Config) -> Result<(), Self::Error> {
        self.properties.max_iterations = config.max_iterations;
        self.properties.iteration_ttl = config.iteration_ttl;
        self.started_at = Some(Instant::now());
        Ok(())
    }

    fn operate(&self) -> Result<Self::Handle, Self::Error> {
        // operate() returns current history; actual iteration happens in maintain()
        Ok(self.history.clone())
    }

    fn terminate(handle: Self::Handle) -> Result<AuditRecord, Self::Error> {
        let fail_count = handle
            .iter()
            .filter(|h| matches!(h.verdict, ExperimentVerdict::Fail { .. }))
            .count() as u32;
        Ok(AuditRecord {
            surface_name: "ralph-loop".into(),
            uptime: handle.iter().map(|h| h.elapsed).sum(),
            exit_reason: format!("{} iterations completed, {fail_count} failed", handle.len()),
            crash_count: handle.iter().filter(|h| h.crashed).count() as u32,
            bytes_logged: 0,
        })
    }

    fn maintain(&self) -> MaintenanceAction {
        if self.should_terminate() {
            return MaintenanceAction::Terminate;
        }
        MaintenanceAction::NoOp
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RalphLoopConfig {
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u32,
    #[serde(default = "default_iteration_ttl")]
    pub iteration_ttl: Duration,
}

fn default_max_iterations() -> u32 {
    10
}

fn default_iteration_ttl() -> Duration {
    Duration::from_secs(300)
}

#[derive(Debug, thiserror::Error)]
pub enum RalphLoopError {
    #[error("ralph loop error: {0}")]
    General(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gated::autoresearch::CargoTestResearcher;

    #[test]
    fn ralph_loop_properties_default() {
        let props = RalphLoopProperties::default();
        assert_eq!(props.max_iterations, 10);
        assert_eq!(props.iteration_ttl, Duration::from_secs(300));
        assert_eq!(props.crash_budget, 3);
    }

    #[test]
    fn ralph_loop_single_iteration() {
        let researcher = CargoTestResearcher::new("test-crate");
        let mut loop_surface = RalphLoopSurface::new(
            researcher,
            RalphLoopProperties {
                max_iterations: 5,
                ..Default::default()
            },
        );
        let verdict = loop_surface.iterate().unwrap();
        assert_eq!(verdict, ExperimentVerdict::Fail {
            reason: "no tests ran".into()
        });
        assert_eq!(loop_surface.history.len(), 1);
    }

    #[test]
    fn ralph_loop_max_iterations() {
        let researcher = CargoTestResearcher::new("test-crate");
        let mut loop_surface = RalphLoopSurface::new(
            researcher,
            RalphLoopProperties {
                max_iterations: 2,
                ..Default::default()
            },
        );
        assert!(loop_surface.iterate().is_ok());
        assert!(loop_surface.iterate().is_ok());
        assert!(loop_surface.iterate().is_err()); // max reached
        assert_eq!(loop_surface.history.len(), 2);
    }

    #[test]
    fn ralph_loop_should_terminate() {
        let researcher = CargoTestResearcher::new("test");
        let mut loop_surface = RalphLoopSurface::new(
            researcher,
            RalphLoopProperties {
                max_iterations: 1,
                crash_budget: 5,
                ..Default::default()
            },
        );
        assert!(!loop_surface.should_terminate());
        let _ = loop_surface.iterate();
        assert!(loop_surface.should_terminate()); // max iterations reached
    }

    #[test]
    fn process_surface_lifecycle() {
        let researcher = CargoTestResearcher::new("test-crate");
        let loop_surface = RalphLoopSurface::new(researcher, RalphLoopProperties::default());
        let cap = loop_surface.capability();
        assert_eq!(cap.name, "ralph-loop");
        assert!(cap.governance.validate().is_ok());
    }

    #[test]
    fn governance_policy_from_properties() {
        let loop_surface = RalphLoopSurface::new(
            CargoTestResearcher::new("test"),
            RalphLoopProperties {
                max_iterations: 5,
                iteration_ttl: Duration::from_secs(60),
                crash_budget: 2,
                cadence: Duration::from_millis(500),
                name: "my-loop".into(),
            },
        );
        let g = loop_surface.governance();
        assert_eq!(g.crash_budget, 2);
        assert_eq!(g.max_ttl, Duration::from_secs(300)); // 5 * 60
        assert!(!g.auto_restart);
    }

    #[test]
    fn terminate_produces_audit() {
        let researcher = CargoTestResearcher::new("test");
        let loop_surface = RalphLoopSurface::<CargoTestResearcher>::new(
            researcher,
            RalphLoopProperties::default(),
        );
        let handle = Vec::new();
        let audit = <RalphLoopSurface::<CargoTestResearcher>>::terminate(handle).unwrap();
        assert_eq!(audit.surface_name, "ralph-loop");
    }

    #[test]
    fn maintain_terminates_at_max() {
        let researcher = CargoTestResearcher::new("test");
        let mut loop_surface = RalphLoopSurface::new(
            researcher,
            RalphLoopProperties {
                max_iterations: 1,
                ..Default::default()
            },
        );
        assert_eq!(loop_surface.maintain(), MaintenanceAction::NoOp);
        let _ = loop_surface.iterate();
        assert_eq!(loop_surface.maintain(), MaintenanceAction::Terminate);
    }
}
