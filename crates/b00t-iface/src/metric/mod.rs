//! Metric — state-machine-observable metric animation subsystem.
//!
//! Every `ProcessSurface` produces a stream of metrics as it transitions
//! through its lifecycle. This module defines:
//!
//! - `MetricSnapshot` — a typed metric value at a point in time
//! - `MetricStream` — a time-ordered sequence of snapshots from one surface
//! - `MetricAnimation` — interpolation between two snapshots for smooth rendering
//! - `MetricRegistry` — collects metrics across all surfaces in a harness
//!
//! # Isometric integration
//!
//! Metrics are rendered as animated isometric panels: each surface gets a
//! 3D card showing its current state, transition count, crash count, and
//! uptime. The card animates between states using SVG `<animateTransform>`.

use serde::Serialize;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// A typed metric value from a surface lifecycle.
#[derive(Debug, Clone, Serialize)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    DurationMs(u64),
    State(String),
}

impl std::fmt::Display for MetricValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Counter(c) => write!(f, "{c}"),
            Self::Gauge(g) => write!(f, "{g:.2}"),
            Self::DurationMs(ms) => write!(f, "{ms}ms"),
            Self::State(s) => write!(f, "{s}"),
        }
    }
}

/// A single metric observation at a point in time.
#[derive(Debug, Clone, Serialize)]
pub struct MetricObservation {
    pub surface: String,
    pub key: String,
    pub value: MetricValue,
    pub timestamp: Duration, // elapsed since harness start
}

/// A time-ordered stream of observations for one metric key.
#[derive(Debug, Clone)]
pub struct MetricStream {
    pub surface: String,
    pub key: String,
    pub observations: Vec<MetricObservation>,
}

impl MetricStream {
    pub fn latest(&self) -> Option<&MetricObservation> {
        self.observations.last()
    }

    pub fn count(&self) -> usize {
        self.observations.len()
    }
}

/// Interpolation between two metric values for animation.
#[derive(Debug, Clone)]
pub struct MetricAnimation {
    pub from: MetricValue,
    pub to: MetricValue,
    pub elapsed: Duration,
    pub duration: Duration,
}

impl MetricAnimation {
    /// Progress as a 0.0–1.0 ratio.
    pub fn progress(&self) -> f64 {
        if self.duration.is_zero() {
            return 1.0;
        }
        (self.elapsed.as_secs_f64() / self.duration.as_secs_f64()).min(1.0)
    }

    /// Whether animation is complete.
    pub fn is_complete(&self) -> bool {
        self.elapsed >= self.duration
    }

    /// Animate a counter: interpolate between from and to values.
    pub fn animated_counter(&self) -> u64 {
        let from = match &self.from {
            MetricValue::Counter(c) => *c,
            _ => 0,
        };
        let to = match &self.to {
            MetricValue::Counter(c) => *c,
            _ => 0,
        };
        let p = self.progress();
        from + ((to as f64 - from as f64) * p) as u64
    }
}

/// Collects metrics across all surfaces during a harness run.
#[derive(Debug, Clone)]
pub struct MetricRegistry {
    pub streams: Vec<MetricStream>,
    pub start: Instant,
    pub animating: Vec<MetricAnimation>,
}

impl MetricRegistry {
    pub fn new() -> Self {
        Self {
            streams: Vec::new(),
            start: Instant::now(),
            animating: Vec::new(),
        }
    }

    /// Record a metric observation.
    pub fn record(&mut self, surface: &str, key: &str, value: MetricValue) {
        let timestamp = self.start.elapsed();
        let obs = MetricObservation {
            surface: surface.to_owned(),
            key: key.to_owned(),
            value,
            timestamp,
        };

        // Find or create stream
        let target_key = key.to_owned();
        let target_surface = surface.to_owned();
        if let Some(stream) = self.streams.iter_mut().find(|s| s.surface == target_surface && s.key == target_key) {
            let prev = stream.latest().map(|o| o.value.clone());
            stream.observations.push(obs);
            if let Some(from) = prev {
                self.animating.push(MetricAnimation {
                    from,
                    to: stream.latest().unwrap().value.clone(),
                    elapsed: Duration::ZERO,
                    duration: Duration::from_millis(500),
                });
            }
        } else {
            self.streams.push(MetricStream {
                surface: surface.to_owned(),
                key: key.to_owned(),
                observations: vec![obs],
            });
        }
    }

    /// Record a set of standard lifecycle metrics from a state machine.
    pub fn record_lifecycle(&mut self, surface: &str, state: &str, crash_count: u32, uptime: Duration) {
        self.record(surface, "state", MetricValue::State(state.to_owned()));
        self.record(surface, "crashes", MetricValue::Counter(crash_count as u64));
        self.record(surface, "uptime_ms", MetricValue::DurationMs(uptime.as_millis() as u64));
    }

    /// Update all animations by the given delta.
    pub fn tick_animations(&mut self, delta: Duration) {
        for anim in &mut self.animating {
            anim.elapsed += delta;
        }
        self.animating.retain(|a| !a.is_complete());
    }

    /// Get all metrics as a flat map (surface → key → display string).
    pub fn flat_display(&self) -> HashMap<String, HashMap<String, String>> {
        let mut map: HashMap<String, HashMap<String, String>> = HashMap::new();
        for stream in &self.streams {
            if let Some(latest) = stream.latest() {
                map.entry(stream.surface.clone())
                    .or_default()
                    .insert(stream.key.clone(), latest.value.to_string());
            }
        }
        map
    }
}

impl Default for MetricRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_and_retrieve() {
        let mut reg = MetricRegistry::new();
        reg.record("watcher", "state", MetricValue::State("running".into()));
        reg.record("watcher", "crashes", MetricValue::Counter(0));
        reg.record_lifecycle("autoresearch", "idle", 0, Duration::from_secs(60));

        let flat = reg.flat_display();
        assert_eq!(flat.get("watcher").unwrap().get("state").unwrap(), "running");
        assert_eq!(flat.get("autoresearch").unwrap().get("uptime_ms").unwrap(), "60000ms");
        // 2 from watcher (state, crashes) + 3 from autoresearch (state, crashes, uptime_ms) = 5 streams
        assert_eq!(reg.streams.len(), 5);
    }

    #[test]
    fn metric_stream_latest() {
        let mut stream = MetricStream {
            surface: "test".into(),
            key: "x".into(),
            observations: vec![],
        };
        assert!(stream.latest().is_none());

        stream.observations.push(MetricObservation {
            surface: "test".into(),
            key: "x".into(),
            value: MetricValue::Counter(1),
            timestamp: Duration::from_secs(1),
        });
        assert_eq!(stream.count(), 1);
        assert!(stream.latest().is_some());
    }

    #[test]
    fn animation_progress() {
        let a = MetricAnimation {
            from: MetricValue::Counter(0),
            to: MetricValue::Counter(100),
            elapsed: Duration::from_millis(250),
            duration: Duration::from_millis(500),
        };
        let p = a.progress();
        assert!((p - 0.5).abs() < 0.01);
        assert!(!a.is_complete());

        let a2 = MetricAnimation {
            elapsed: Duration::from_millis(500),
            ..a
        };
        assert!(a2.is_complete());
    }

    #[test]
    fn animated_counter_interpolation() {
        let mut a = MetricAnimation {
            from: MetricValue::Counter(10),
            to: MetricValue::Counter(20),
            elapsed: Duration::ZERO,
            duration: Duration::from_millis(1000),
        };
        assert_eq!(a.animated_counter(), 10);
        a.elapsed = Duration::from_millis(500);
        assert_eq!(a.animated_counter(), 15);
        a.elapsed = Duration::from_millis(1000);
        assert_eq!(a.animated_counter(), 20);
    }

    #[test]
    fn tick_animations_removes_completed() {
        let mut reg = MetricRegistry::new();
        reg.record("s", "c", MetricValue::Counter(0));
        reg.record("s", "c", MetricValue::Counter(10));
        assert_eq!(reg.animating.len(), 1);
        reg.tick_animations(Duration::from_secs(10));
        assert_eq!(reg.animating.len(), 0);
    }

    #[test]
    fn metric_value_display() {
        assert_eq!(MetricValue::Counter(42).to_string(), "42");
        assert_eq!(MetricValue::State("running".into()).to_string(), "running");
        assert_eq!(MetricValue::DurationMs(5000).to_string(), "5000ms");
        assert!(!MetricValue::Gauge(3.14).to_string().is_empty());
    }
}
