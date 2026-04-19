//! Pipeline Visualization: AST-driven, deterministic Mermaid generation.
//! Generates animated flow charts directly from workflow execution state.
//!
//! ## Design
//! 1. AST is the TOML workflow definition (WorkflowToml)
//! 2. Kasuari provides layout constraints for positioning  
//! 3. Mermaid stateDiagram-v2 generates with animation metadata
//! 4. Each state shows real-time status from PipelineState

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::pipeline::State as PipelineStateEnum;
use crate::validation::Disposition;

/// Visual state for a node in the graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeVisualState {
    /// Waiting, not yet entered.
    Idle,
    /// Currently active.
    Active,
    /// Successfully completed (green).
    Success,
    /// Failed with recoverable error.
    Warning,
    /// Failed with unrecoverable error (red).
    Error,
    /// Awaiting human review.
    Review,
}

impl NodeVisualState {
    pub fn from_pipeline(state: PipelineStateEnum, confidence: f32, issues: &[crate::validation::Issue]) -> Self {
        if issues.iter().any(|i| i.disposition == Disposition::Unrecoverable) {
            return NodeVisualState::Error;
        }
        match state {
            PipelineStateEnum::Committed => NodeVisualState::Success,
            PipelineStateEnum::NeedsReview => NodeVisualState::Review,
            PipelineStateEnum::Ingested |
            PipelineStateEnum::Validating |
            PipelineStateEnum::Classifying |
            PipelineStateEnum::Reconciling => {
                if confidence < 0.5 {
                    NodeVisualState::Warning
                } else {
                    NodeVisualState::Active
                }
            }
        }
    }

    /// Get Mermaid color for this state.
    pub fn fill(&self) -> &'static str {
        match self {
            NodeVisualState::Idle => "#f0f0f0",
            NodeVisualState::Active => "#4a90d9",
            NodeVisualState::Success => "#4caf50",
            NodeVisualState::Warning => "#ff9800",
            NodeVisualState::Error => "#f44336",
            NodeVisualState::Review => "#9c27b0",
        }
    }

    /// Get animation class for transitions.
    pub fn animation_class(&self) -> &'static str {
        match self {
            NodeVisualState::Active => "pulse",
            NodeVisualState::Success => "check",
            NodeVisualState::Warning => "shake",
            NodeVisualState::Error => "blink",
            NodeVisualState::Review => "bounce",
            NodeVisualState::Idle => "",
        }
    }
}

/// Edge visual representing a transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeVisual {
    pub from: String,
    pub to: String,
    pub label: String,
    pub active: bool,
    pub weight: f32,
}

impl EdgeVisual {
    pub fn new(from: impl Into<String>, to: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            label: label.into(),
            active: false,
            weight: 1.0,
        }
    }

    pub fn with_weight(mut self, w: f32) -> Self {
        self.weight = w;
        self
    }
}

/// Complete graph visualization data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineGraph {
    pub nodes: HashMap<String, NodeVisualState>,
    pub edges: Vec<EdgeVisual>,
    pub current_state: String,
    pub accumulated_confidence: f32,
}

impl Default for PipelineGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelineGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            current_state: "Ingested".to_string(),
            accumulated_confidence: 1.0,
        }
    }

    /// Update with current pipeline execution state.
    pub fn update(
        &mut self,
        current: PipelineStateEnum,
        confidence: f32,
        issues: &[crate::validation::Issue],
    ) {
        let state_name = match current {
            PipelineStateEnum::Ingested => "Ingested",
            PipelineStateEnum::Validating => "Validating",
            PipelineStateEnum::Classifying => "Classifying",
            PipelineStateEnum::Reconciling => "Reconciling",
            PipelineStateEnum::Committed => "Committed",
            PipelineStateEnum::NeedsReview => "NeedsReview",
        };

        self.current_state = state_name.to_string();
        self.accumulated_confidence = confidence;

        // Update all nodes' visual state based on current position
        for (node_name, visual_state) in self.nodes.iter_mut() {
            *visual_state = NodeVisualState::from_pipeline(
                state_from_name(node_name),
                confidence,
                issues,
            );
        }

        // Mark current node as active (overrides above)
        if let Some(state) = self.nodes.get_mut(state_name) {
            *state = NodeVisualState::from_pipeline(current, confidence, issues);
        }
    }

    /// Generate animated Mermaid from current state.
    pub fn to_mermaid(&self) -> String {
        let mut diagram = String::from("stateDiagram-v2\n");
        diagram.push_str("    direction LR\n\n");

        // Define states with styling
        for (name, visual) in &self.nodes {
            let fill = visual.fill();
            let anim_class = visual.animation_class();
            if !anim_class.is_empty() {
                diagram.push_str(&format!(
                    "    state {} {{\n      {}: {}:::{}\n    }}\n",
                    name, name, fill, anim_class
                ));
            } else {
                diagram.push_str(&format!("    state {} {{\n      {}: {}\n    }}\n", name, name, fill));
            }
        }

        // Transitions with animation
        for edge in &self.edges {
            let arrow = if edge.active { "-->!" } else { "-->" };
            diagram.push_str(&format!(
                "    {} {} {} : {}\n",
                edge.from, arrow, edge.to, edge.label
            ));
        }

        // Current state marker
        diagram.push_str(&format!("\n    [*] --> {}\n", self.current_state));

        diagram
    }

    /// Generate CSS animation styles.
    pub fn animation_styles(&self) -> String {
        r#"
@keyframes pulse {
    0% { transform: scale(1); }
    50% { transform: scale(1.1); }
    100% { transform: scale(1); }
}
@keyframes check {
    0% { fill: #4a90d9; }
    100% { fill: #4caf50; }
}
@keyframes shake {
    0%, 100% { transform: translateX(0); }
    25% { transform: translateX(-5px); }
    75% { transform: translateX(5px); }
}
@keyframes blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.3; }
}
@keyframes bounce {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-10px); }
}
.pulse { animation: pulse 1s infinite; }
.check { animation: check 0.5s forwards; }
.shake { animation: shake 0.3s infinite; }
.blink { animation: blink 0.5s infinite; }
.bounce { animation: bounce 0.5s infinite; }
"#.to_string()
    }
}

/// Helper: convert state name to enum.
fn state_from_name(name: &str) -> PipelineStateEnum {
    match name {
        "Ingested" => PipelineStateEnum::Ingested,
        "Validating" => PipelineStateEnum::Validating,
        "Classifying" => PipelineStateEnum::Classifying,
        "Reconciling" => PipelineStateEnum::Reconciling,
        "Committed" => PipelineStateEnum::Committed,
        "NeedsReview" => PipelineStateEnum::NeedsReview,
        _ => PipelineStateEnum::Ingested,
    }
}

/// Generate layout constraints using Kasuari.
pub mod layout {
    use super::*;

    /// Kasuari-backed layout constraint solver.
    pub struct LayoutSolver;

    impl Default for LayoutSolver {
        fn default() -> Self {
            Self
        }
    }

    impl LayoutSolver {
        pub fn new() -> Self {
            Self
        }

        /// Generate layout constraints for a pipeline graph.
        /// Returns (x, width) for each state.
        pub fn generate_layout(&self, graph: &PipelineGraph) -> HashMap<String, (f32, f32)> {
            let mut result = HashMap::new();
            let mut x = 100.0;

            let layer_order = ["Ingested", "Validating", "Classifying", "Reconciling", "Committed", "NeedsReview"];
            for state in layer_order {
                if graph.nodes.contains_key(state) {
                    let width = if state == &graph.current_state { 120.0 } else { 100.0 };
                    result.insert(state.to_string(), (x, width));
                    x += 150.0;
                }
            }

            result
        }
    }
}

/// Generate complete HTML visualization.
pub fn to_html(graph: &PipelineGraph) -> String {
    let mermaid = graph.to_mermaid();
    let styles = graph.animation_styles();
    let current = &graph.current_state;
    let confidence = graph.accumulated_confidence;

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Pipeline: {}</title>
    <script src="https://cdn.jsdelivr.net/npm/mermaid/dist/mermaid.min.js"></script>
    <style>
        body {{ font-family: system-ui, sans-serif; padding: 20px; }}
        #graph {{ display: flex; justify-content: center; }}
        .status {{ margin: 20px; text-align: center; }}
        .confidence {{ font-size: 2em; font-weight: bold; color: {} }}
        {}
    </style>
</head>
<body>
    <div class="status">
        <h1>Pipeline: {}</h1>
        <div class="confidence">{:.2}</div>
    </div>
    <div id="graph" class="mermaid">
{}
    </div>
    <script>
        mermaid.initialize({{ startOnLoad: true }});
    </script>
</body>
</html>"#,
        current,
        if confidence > 0.7 { "#4caf50" } else if confidence > 0.4 { "#ff9800" } else { "#f44336" },
        styles,
        current,
        confidence,
        mermaid
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_visual_state() {
        // Test Active state
        let state = NodeVisualState::from_pipeline(
            PipelineStateEnum::Validating,
            0.9,
            &[]
        );
        assert_eq!(state, NodeVisualState::Active);

        // Test Warning (low confidence)
        let state = NodeVisualState::from_pipeline(
            PipelineStateEnum::Classifying,
            0.3,
            &[]
        );
        assert_eq!(state, NodeVisualState::Warning);
    }

    #[test]
    fn test_mermaid_generation() {
        let mut graph = PipelineGraph::new();
        graph.nodes.insert("Ingested".to_string(), NodeVisualState::Success);
        graph.nodes.insert("Validating".to_string(), NodeVisualState::Active);
        graph.nodes.insert("Classifying".to_string(), NodeVisualState::Idle);
        graph.current_state = "Validating".to_string();
        graph.accumulated_confidence = 0.85;

        let mermaid = graph.to_mermaid();
        assert!(mermaid.contains("stateDiagram-v2"));
        assert!(mermaid.contains("Validating"));
    }

    #[test]
    fn test_html_generation() {
        let graph = PipelineGraph {
            nodes: HashMap::from([
                ("Ingested".to_string(), NodeVisualState::Success),
                ("Validating".to_string(), NodeVisualState::Active),
            ]),
            edges: vec![],
            current_state: "Validating".to_string(),
            accumulated_confidence: 0.9,
        };

        let html = to_html(&graph);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("mermaid"));
    }

    #[test]
    fn test_layout_constraints() {
        let solver = layout::LayoutSolver::new();
        let mut graph = PipelineGraph::new();
        graph.nodes.insert("Validating".to_string(), NodeVisualState::Active);
        graph.current_state = "Validating".to_string();
        graph.accumulated_confidence = 0.8;

        let layout = solver.generate_layout(&graph);
        assert!(!layout.is_empty());
    }
}