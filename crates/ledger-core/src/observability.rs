//! Internal OpenTelemetry/Rotel surface for l3dg3rr.
//!
//! This module deliberately models the OpenTelemetry object shapes that l3dg3rr
//! needs before choosing a collector transport. Rotel remains the embedded
//! collector boundary; classification and audit justification stay typed here.

use std::collections::BTreeMap;

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OTelAnyValue {
    String(String),
    Bool(bool),
    I64(i64),
    F64(f64),
    Bytes(Vec<u8>),
    Array(Vec<OTelAnyValue>),
    KvList(BTreeMap<String, OTelAnyValue>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OTelKeyValue {
    pub key: String,
    pub value: OTelAnyValue,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OTelResource {
    pub attributes: BTreeMap<String, String>,
    pub schema_url: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OTelInstrumentationScope {
    pub name: String,
    pub version: Option<String>,
    pub attributes: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OTelSignal {
    Log,
    Metric,
    Trace,
}

impl OTelSignal {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Log => "log",
            Self::Metric => "metric",
            Self::Trace => "trace",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OTelSeverityNumber {
    Trace = 1,
    Debug = 5,
    Info = 9,
    Warn = 13,
    Error = 17,
    Fatal = 21,
}

impl OTelSeverityNumber {
    fn as_str(self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
            Self::Fatal => "FATAL",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OTelLogRecord {
    pub time_unix_nano: u64,
    pub observed_time_unix_nano: u64,
    pub severity_number: OTelSeverityNumber,
    pub severity_text: String,
    pub body: String,
    pub attributes: BTreeMap<String, String>,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
}

impl OTelLogRecord {
    pub fn new(
        time_unix_nano: u64,
        severity_number: OTelSeverityNumber,
        body: impl Into<String>,
    ) -> Self {
        Self {
            time_unix_nano,
            observed_time_unix_nano: time_unix_nano,
            severity_number,
            severity_text: severity_number.as_str().to_string(),
            body: body.into(),
            attributes: BTreeMap::new(),
            trace_id: None,
            span_id: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OTelSpanKind {
    Internal,
    Server,
    Client,
    Producer,
    Consumer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OTelSpanEvent {
    pub name: String,
    pub time_unix_nano: u64,
    pub attributes: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OTelSpan {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub name: String,
    pub kind: OTelSpanKind,
    pub start_time_unix_nano: u64,
    pub end_time_unix_nano: u64,
    pub attributes: BTreeMap<String, String>,
    pub events: Vec<OTelSpanEvent>,
    pub status_code: Option<String>,
    pub status_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OTelMetricData {
    Gauge {
        value: f64,
    },
    Sum {
        value: f64,
        monotonic: bool,
    },
    Histogram {
        count: u64,
        sum: f64,
        bucket_counts: Vec<u64>,
        explicit_bounds: Vec<f64>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OTelMetric {
    pub name: String,
    pub description: String,
    pub unit: String,
    pub time_unix_nano: u64,
    pub attributes: BTreeMap<String, String>,
    pub data: OTelMetricData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OTelEnvelope {
    Log(OTelLogRecord),
    Metric(OTelMetric),
    Trace(OTelSpan),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogShapeRule {
    pub rule_id: String,
    pub abstract_regex_type: String,
    pub pattern: String,
    pub metric_name: String,
    pub metric_delta: i64,
    pub min_severity: OTelSeverityNumber,
    pub rationale: String,
}

impl LogShapeRule {
    pub fn validate(&self) -> Result<(), ObservabilityError> {
        if self.rule_id.trim().is_empty() {
            return Err(ObservabilityError::InvalidRule(
                "rule_id is required".to_string(),
            ));
        }
        if self.abstract_regex_type.trim().is_empty() {
            return Err(ObservabilityError::InvalidRule(
                "abstract_regex_type is required".to_string(),
            ));
        }
        if self.metric_name.trim().is_empty() {
            return Err(ObservabilityError::InvalidRule(
                "metric_name is required".to_string(),
            ));
        }
        Regex::new(&self.pattern)
            .map(|_| ())
            .map_err(|e| ObservabilityError::InvalidRule(format!("invalid regex: {e}")))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricJustification {
    pub rule_id: String,
    pub metric_name: String,
    pub metric_delta: i64,
    pub reason: String,
    pub evidence_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassifiedJournalArtifact {
    pub artifact_id: String,
    pub signal: OTelSignal,
    pub abstract_regex_type: String,
    pub source_time_unix_nano: u64,
    pub severity_text: String,
    pub matched_excerpt: String,
    pub justification: MetricJustification,
    pub attributes: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default)]
pub struct LogShapeClassifier {
    compiled: Vec<(LogShapeRule, Regex)>,
}

impl LogShapeClassifier {
    pub fn new(rules: Vec<LogShapeRule>) -> Result<Self, ObservabilityError> {
        let mut compiled = Vec::with_capacity(rules.len());
        for rule in rules {
            rule.validate()?;
            let regex = Regex::new(&rule.pattern)
                .map_err(|e| ObservabilityError::InvalidRule(format!("invalid regex: {e}")))?;
            compiled.push((rule, regex));
        }
        Ok(Self { compiled })
    }

    pub fn classify_log(&self, log: &OTelLogRecord) -> Vec<ClassifiedJournalArtifact> {
        self.compiled
            .iter()
            .filter(|(rule, _)| log.severity_number >= rule.min_severity)
            .filter_map(|(rule, regex)| {
                let capture = regex.find(&log.body)?;
                Some(classified_artifact(rule, log, capture.as_str()))
            })
            .collect()
    }
}

fn classified_artifact(
    rule: &LogShapeRule,
    log: &OTelLogRecord,
    matched_excerpt: &str,
) -> ClassifiedJournalArtifact {
    let evidence = format!(
        "{}|{}|{}|{}",
        rule.rule_id, rule.abstract_regex_type, log.time_unix_nano, log.body
    );
    let evidence_digest = blake3::hash(evidence.as_bytes()).to_hex().to_string();
    let artifact_identity = format!(
        "classified-journal|{}|{}|{}",
        rule.rule_id, evidence_digest, rule.metric_name
    );

    let mut attributes = log.attributes.clone();
    attributes.insert(
        "otel.signal".to_string(),
        OTelSignal::Log.as_str().to_string(),
    );
    attributes.insert("otel.severity_text".to_string(), log.severity_text.clone());
    if let Some(trace_id) = &log.trace_id {
        attributes.insert("otel.trace_id".to_string(), trace_id.clone());
    }
    if let Some(span_id) = &log.span_id {
        attributes.insert("otel.span_id".to_string(), span_id.clone());
    }

    ClassifiedJournalArtifact {
        artifact_id: blake3::hash(artifact_identity.as_bytes())
            .to_hex()
            .to_string(),
        signal: OTelSignal::Log,
        abstract_regex_type: rule.abstract_regex_type.clone(),
        source_time_unix_nano: log.time_unix_nano,
        severity_text: log.severity_text.clone(),
        matched_excerpt: matched_excerpt.to_string(),
        justification: MetricJustification {
            rule_id: rule.rule_id.clone(),
            metric_name: rule.metric_name.clone(),
            metric_delta: rule.metric_delta,
            reason: rule.rationale.clone(),
            evidence_digest,
        },
        attributes,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelemetryArrowRow {
    pub artifact_id: String,
    pub signal: String,
    pub abstract_regex_type: String,
    pub metric_name: String,
    pub metric_delta: i64,
    pub severity_text: String,
    pub matched_excerpt: String,
    pub justification_digest: String,
    pub source_time_unix_nano: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelemetryArrowBatch {
    pub rows: Vec<TelemetryArrowRow>,
}

impl TelemetryArrowBatch {
    pub fn from_artifacts(artifacts: &[ClassifiedJournalArtifact]) -> Self {
        Self {
            rows: artifacts
                .iter()
                .map(|artifact| TelemetryArrowRow {
                    artifact_id: artifact.artifact_id.clone(),
                    signal: artifact.signal.as_str().to_string(),
                    abstract_regex_type: artifact.abstract_regex_type.clone(),
                    metric_name: artifact.justification.metric_name.clone(),
                    metric_delta: artifact.justification.metric_delta,
                    severity_text: artifact.severity_text.clone(),
                    matched_excerpt: artifact.matched_excerpt.clone(),
                    justification_digest: artifact.justification.evidence_digest.clone(),
                    source_time_unix_nano: artifact.source_time_unix_nano,
                })
                .collect(),
        }
    }

    pub fn classification_columns() -> &'static [&'static str] {
        &[
            "artifact_id",
            "signal",
            "abstract_regex_type",
            "metric_name",
            "metric_delta",
            "severity_text",
            "matched_excerpt",
            "justification_digest",
            "source_time_unix_nano",
        ]
    }

    #[cfg(feature = "otel-arrow")]
    pub fn arrow_schema() -> arrow::datatypes::Schema {
        use arrow::datatypes::{DataType, Field, Schema};

        Schema::new(vec![
            Field::new("artifact_id", DataType::Utf8, false),
            Field::new("signal", DataType::Utf8, false),
            Field::new("abstract_regex_type", DataType::Utf8, false),
            Field::new("metric_name", DataType::Utf8, false),
            Field::new("metric_delta", DataType::Int64, false),
            Field::new("severity_text", DataType::Utf8, false),
            Field::new("matched_excerpt", DataType::Utf8, false),
            Field::new("justification_digest", DataType::Utf8, false),
            Field::new("source_time_unix_nano", DataType::UInt64, false),
        ])
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RotelEndpoint {
    pub otlp_http_endpoint: String,
    pub otlp_grpc_endpoint: String,
    pub arrow_connector_enabled: bool,
}

impl Default for RotelEndpoint {
    fn default() -> Self {
        Self {
            otlp_http_endpoint: "http://127.0.0.1:4318".to_string(),
            otlp_grpc_endpoint: "http://127.0.0.1:4317".to_string(),
            arrow_connector_enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RotelExportPlan {
    pub logs_url: String,
    pub metrics_url: String,
    pub traces_url: String,
    pub arrow_connector_enabled: bool,
    pub arrow_columns: Vec<String>,
}

impl RotelExportPlan {
    pub fn from_endpoint(endpoint: &RotelEndpoint) -> Self {
        let base = endpoint.otlp_http_endpoint.trim_end_matches('/');
        Self {
            logs_url: format!("{base}/v1/logs"),
            metrics_url: format!("{base}/v1/metrics"),
            traces_url: format!("{base}/v1/traces"),
            arrow_connector_enabled: endpoint.arrow_connector_enabled,
            arrow_columns: TelemetryArrowBatch::classification_columns()
                .iter()
                .map(|column| (*column).to_string())
                .collect(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ObservabilityError {
    #[error("invalid observability rule: {0}")]
    InvalidRule(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gpu_fault_rule() -> LogShapeRule {
        LogShapeRule {
            rule_id: "gpu-driver-device-disappeared".to_string(),
            abstract_regex_type: "hardware.gpu.driver.device_handle_unknown".to_string(),
            pattern: "Unable to determine the device handle for GPU[0-9]+.*Unknown Error"
                .to_string(),
            metric_name: "l3dg3rr.hardware.gpu.driver_faults".to_string(),
            metric_delta: 1,
            min_severity: OTelSeverityNumber::Error,
            rationale: "GPU was expected but nvidia-smi returned an unknown device-handle error"
                .to_string(),
        }
    }

    #[test]
    fn classifies_log_shape_into_journal_artifact_and_metric_trigger() {
        let classifier = LogShapeClassifier::new(vec![gpu_fault_rule()]).unwrap();
        let mut log = OTelLogRecord::new(
            1_777_724_525,
            OTelSeverityNumber::Error,
            "Unable to determine the device handle for GPU0: 0000:01:00.0: Unknown Error",
        );
        log.attributes
            .insert("host.name".to_string(), "sm3llsl1k3s0ld3r".to_string());

        let artifacts = classifier.classify_log(&log);

        assert_eq!(artifacts.len(), 1);
        assert_eq!(
            artifacts[0].abstract_regex_type,
            "hardware.gpu.driver.device_handle_unknown"
        );
        assert_eq!(
            artifacts[0].justification.metric_name,
            "l3dg3rr.hardware.gpu.driver_faults"
        );
        assert_eq!(artifacts[0].justification.metric_delta, 1);
        assert_eq!(
            artifacts[0].attributes.get("host.name").map(String::as_str),
            Some("sm3llsl1k3s0ld3r")
        );
    }

    #[test]
    fn classified_artifact_id_is_deterministic() {
        let classifier = LogShapeClassifier::new(vec![gpu_fault_rule()]).unwrap();
        let log = OTelLogRecord::new(
            42,
            OTelSeverityNumber::Error,
            "Unable to determine the device handle for GPU0: 0000:01:00.0: Unknown Error",
        );

        let first = classifier.classify_log(&log);
        let second = classifier.classify_log(&log);

        assert_eq!(first[0].artifact_id, second[0].artifact_id);
        assert_eq!(
            first[0].justification.evidence_digest,
            second[0].justification.evidence_digest
        );
    }

    #[test]
    fn telemetry_arrow_batch_uses_stable_classification_columns() {
        let classifier = LogShapeClassifier::new(vec![gpu_fault_rule()]).unwrap();
        let log = OTelLogRecord::new(
            7,
            OTelSeverityNumber::Fatal,
            "Unable to determine the device handle for GPU0: 0000:01:00.0: Unknown Error",
        );
        let artifacts = classifier.classify_log(&log);
        let batch = TelemetryArrowBatch::from_artifacts(&artifacts);

        assert_eq!(batch.rows.len(), 1);
        assert_eq!(
            TelemetryArrowBatch::classification_columns(),
            &[
                "artifact_id",
                "signal",
                "abstract_regex_type",
                "metric_name",
                "metric_delta",
                "severity_text",
                "matched_excerpt",
                "justification_digest",
                "source_time_unix_nano",
            ]
        );
        assert_eq!(batch.rows[0].signal, "log");
    }

    #[test]
    fn rotel_export_plan_keeps_otlp_and_arrow_connector_explicit() {
        let endpoint = RotelEndpoint::default();
        let plan = RotelExportPlan::from_endpoint(&endpoint);

        assert_eq!(plan.logs_url, "http://127.0.0.1:4318/v1/logs");
        assert_eq!(plan.metrics_url, "http://127.0.0.1:4318/v1/metrics");
        assert_eq!(plan.traces_url, "http://127.0.0.1:4318/v1/traces");
        assert!(plan.arrow_connector_enabled);
        assert!(plan
            .arrow_columns
            .iter()
            .any(|c| c == "abstract_regex_type"));
    }
}
