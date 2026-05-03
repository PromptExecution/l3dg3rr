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
    pub fn from_pipeline(
        state: PipelineStateEnum,
        confidence: f32,
        issues: &[crate::validation::Issue],
    ) -> Self {
        if issues
            .iter()
            .any(|i| i.disposition == Disposition::Unrecoverable)
        {
            return NodeVisualState::Error;
        }
        match state {
            PipelineStateEnum::Committed => NodeVisualState::Success,
            PipelineStateEnum::NeedsReview => NodeVisualState::Review,
            PipelineStateEnum::Ingested
            | PipelineStateEnum::Validating
            | PipelineStateEnum::Classifying
            | PipelineStateEnum::Reconciling => {
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
            *visual_state =
                NodeVisualState::from_pipeline(state_from_name(node_name), confidence, issues);
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
                diagram.push_str(&format!(
                    "    state {} {{\n      {}: {}\n    }}\n",
                    name, name, fill
                ));
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

    /// Generate animated inline SVG with optional SMIL reflow.
    /// When `previous` is `Some`, diffs node positions and adds
    /// `<animateTransform>` for nodes whose position changed.
    pub fn to_animated_svg(&self, previous: Option<&PipelineGraph>) -> String {
        let solver = layout::LayoutSolver::new();
        let layout = solver.generate_layout(self);

        let node_height = 50.0_f32;
        let svg_height = 200.0_f32;
        let padding = 40.0_f32;

        let svg_width = layout.values().map(|(x, w)| x + w).fold(0.0_f32, f32::max) + padding;

        let prev_layout = previous.map(|p| solver.generate_layout(p));
        let dur = 300u32;

        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {:.0} {:.0}" width="100%" height="auto">
  <defs>
    {}  </defs>
"#,
            svg_width.ceil(),
            svg_height,
            self.svg_marker_defs(),
        );

        // edges first (below nodes)
        for edge in &self.edges {
            let from_pos = layout.get(&edge.from);
            let to_pos = layout.get(&edge.to);
            if let (Some(&(fx, fw)), Some(&(tx, _tw))) = (from_pos, to_pos) {
                let x1 = fx + fw;
                let y1 = svg_height / 2.0;
                let x2 = tx;
                let y2 = svg_height / 2.0;
                let label_x = (x1 + x2) / 2.0;
                let stroke_color = "#666";
                svg.push_str(&format!(
                    r#"  <line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{s}" stroke-width="2" marker-end="url(#arrowhead)" />
  <text x="{:.1}" y="{:.1}" text-anchor="middle" fill="{s}" font-size="10">{}</text>
"#,
                    x1, y1, x2, y2,
                    label_x, y1 - 6.0,
                    crate::iso::xml_attr_escape(&edge.label),
                    s = stroke_color,
                ));
            }
        }

        // nodes
        let node_names: Vec<String> = {
            let mut names: Vec<&String> = self.nodes.keys().collect();
            names.sort();
            names.into_iter().cloned().collect()
        };
        for name in &node_names {
            let visual = &self.nodes[name];
            let pos = layout.get(name);
            if let Some(&(x, w)) = pos {
                let fill = visual.fill();
                let anim_class = visual.animation_class();
                let y = (svg_height - node_height) / 2.0;
                let rx = 6.0_f32;

                let mut group = String::new();
                // animation only on nodes present in both layouts with different x
                if let Some(ref pl) = prev_layout {
                    if let Some(&(px, _pw)) = pl.get(name) {
                        if (px - x).abs() > 0.5 {
                            let text_fill = "#fff";
                            let font_family = "sans-serif";
                            group.push_str(&format!(
                                r#"  <g>
    <rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{nh:.1}" rx="{rx:.1}" fill="{fill}" class="{cls}">
      <animateTransform attributeName="transform" type="translate"
        from="{dx:.1} 0" to="0 0" dur="{dur}ms" fill="freeze" />
    </rect>
    <text x="{tx:.1}" y="{ty:.1}" text-anchor="middle" fill="{tf}" font-size="12" font-family="{ff}">
      <animateTransform attributeName="transform" type="translate"
        from="{dx:.1} 0" to="0 0" dur="{dur}ms" fill="freeze" /><tspan>{label}</tspan>
    </text>
  </g>
"#,
                                x = x, y = y, w = w, nh = node_height, rx = rx,
                                fill = fill, cls = anim_class,
                                dx = px - x, dur = dur,
                                tx = x + w / 2.0, ty = y + node_height / 2.0 + 4.0,
                                tf = text_fill, ff = font_family,
                                label = crate::iso::xml_attr_escape(name),
                            ));
                            svg.push_str(&group);
                            continue;
                        }
                    }
                }

                // static node (no animation)
                let text_fill = "#fff";
                let font_family = "sans-serif";
                group.push_str(&format!(
                    r#"  <rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{nh:.1}" rx="{rx:.1}" fill="{fill}" class="{cls}" />
  <text x="{tx:.1}" y="{ty:.1}" text-anchor="middle" fill="{tf}" font-size="12" font-family="{ff}">{label}</text>
"#,
                    x = x, y = y, w = w, nh = node_height, rx = rx,
                    fill = fill, cls = anim_class,
                    tx = x + w / 2.0, ty = y + node_height / 2.0 + 4.0,
                    tf = text_fill, ff = font_family,
                    label = crate::iso::xml_attr_escape(name),
                ));
                svg.push_str(&group);
            }
        }

        svg.push_str("</svg>\n");
        svg
    }

    fn svg_marker_defs(&self) -> String {
        let fill = "#666";
        format!(
            r#"    <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="10" refY="3.5" orient="auto">
      <polygon points="0 0, 10 3.5, 0 7" fill="{fill}" />
    </marker>
"#,
            fill = fill,
        )
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
"#
        .to_string()
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
    use kasuari::WeightedRelation::*;
    use kasuari::{Solver, Strength, Variable};

    /// Kasuari-backed layout constraint solver.
    pub struct LayoutSolver {
        gap: f64,
        default_width: f64,
        current_width: f64,
        start_x: f64,
    }

    impl Default for LayoutSolver {
        fn default() -> Self {
            Self {
                gap: 150.0,
                default_width: 100.0,
                current_width: 120.0,
                start_x: 100.0,
            }
        }
    }

    impl LayoutSolver {
        pub fn new() -> Self {
            Self::default()
        }

        /// Generate layout constraints for a pipeline graph.
        /// Returns (x, width) for each state.
        pub fn generate_layout(&self, graph: &PipelineGraph) -> HashMap<String, (f32, f32)> {
            let layer_order = [
                "Ingested",
                "Validating",
                "Classifying",
                "Reconciling",
                "Committed",
                "NeedsReview",
            ];

            let nodes: Vec<&str> = layer_order
                .iter()
                .filter(|s| graph.nodes.contains_key(**s))
                .copied()
                .collect();

            if nodes.is_empty() {
                return HashMap::new();
            }

            let mut solver = Solver::new();
            let mut x_vars: HashMap<&str, Variable> = HashMap::new();
            let mut w_vars: HashMap<&str, Variable> = HashMap::new();

            for &name in &nodes {
                let x = Variable::new();
                let w = Variable::new();
                x_vars.insert(name, x);
                w_vars.insert(name, w);
            }

            for &name in &nodes {
                let x = x_vars[name];
                let w = w_vars[name];
                let is_current = name == graph.current_state;
                let pref_width = if is_current {
                    self.current_width
                } else {
                    self.default_width
                };

                // positive width
                solver.add_constraint(w | GE(Strength::REQUIRED) | 0.0).ok();
                // positive x
                solver.add_constraint(x | GE(Strength::REQUIRED) | 0.0).ok();
                // preferred width (STAY: weak so ordering wins if conflict)
                solver
                    .add_constraint(w | EQ(Strength::WEAK) | pref_width)
                    .ok();
            }

            // sequential ordering: x_i + w_i + gap <= x_{i+1}
            for pair in nodes.windows(2) {
                let left_x = x_vars[pair[0]];
                let left_w = w_vars[pair[0]];
                let right_x = x_vars[pair[1]];

                let left_edge = left_x + left_w;
                solver
                    .add_constraint((left_edge + self.gap) | LE(Strength::REQUIRED) | right_x)
                    .ok();
            }

            // first node starting position (STAY: weak preference)
            if let Some(&first) = nodes.first() {
                solver
                    .add_constraint(x_vars[first] | EQ(Strength::WEAK) | self.start_x)
                    .ok();
            }

            let mut result = HashMap::new();
            for &name in &nodes {
                let x = solver.get_value(x_vars[name]) as f32;
                let w = solver.get_value(w_vars[name]) as f32;
                result.insert(name.to_string(), (x, w));
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
        if confidence > 0.7 {
            "#4caf50"
        } else if confidence > 0.4 {
            "#ff9800"
        } else {
            "#f44336"
        },
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
        let state = NodeVisualState::from_pipeline(PipelineStateEnum::Validating, 0.9, &[]);
        assert_eq!(state, NodeVisualState::Active);

        // Test Warning (low confidence)
        let state = NodeVisualState::from_pipeline(PipelineStateEnum::Classifying, 0.3, &[]);
        assert_eq!(state, NodeVisualState::Warning);
    }

    #[test]
    fn test_mermaid_generation() {
        let mut graph = PipelineGraph::new();
        graph
            .nodes
            .insert("Ingested".to_string(), NodeVisualState::Success);
        graph
            .nodes
            .insert("Validating".to_string(), NodeVisualState::Active);
        graph
            .nodes
            .insert("Classifying".to_string(), NodeVisualState::Idle);
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
        graph
            .nodes
            .insert("Ingested".to_string(), NodeVisualState::Success);
        graph
            .nodes
            .insert("Validating".to_string(), NodeVisualState::Active);
        graph
            .nodes
            .insert("Classifying".to_string(), NodeVisualState::Idle);
        graph
            .nodes
            .insert("Committed".to_string(), NodeVisualState::Idle);
        graph.current_state = "Validating".to_string();
        graph.accumulated_confidence = 0.8;

        let layout = solver.generate_layout(&graph);
        assert!(!layout.is_empty());

        // verify ordering: Ingested < Validating < Classifying < Committed
        let ingested = layout.get("Ingested").unwrap();
        let validating = layout.get("Validating").unwrap();
        let classifying = layout.get("Classifying").unwrap();
        let committed = layout.get("Committed").unwrap();

        let gap = 150.0;

        // x positions must be strictly increasing
        assert!(ingested.0 < validating.0, "Ingested.x < Validating.x");
        assert!(validating.0 < classifying.0, "Validating.x < Classifying.x");
        assert!(classifying.0 < committed.0, "Classifying.x < Committed.x");

        // no overlap: left_edge + gap <= right_x
        assert!(
            ingested.0 + ingested.1 + gap <= validating.0 + 0.01,
            "no overlap Ingested -> Validating"
        );
        assert!(
            validating.0 + validating.1 + gap <= classifying.0 + 0.01,
            "no overlap Validating -> Classifying"
        );
        assert!(
            classifying.0 + classifying.1 + gap <= committed.0 + 0.01,
            "no overlap Classifying -> Committed"
        );

        // current node (Validating) gets width ~120.0, others ~100.0
        assert!((validating.1 - 120.0).abs() < 1.0, "Validating width ~120");
        assert!((ingested.1 - 100.0).abs() < 1.0, "Ingested width ~100");
        assert!((committed.1 - 100.0).abs() < 1.0, "Committed width ~100");
    }

    #[test]
    fn test_animated_svg_static() {
        let mut graph = PipelineGraph::new();
        graph
            .nodes
            .insert("Ingested".to_string(), NodeVisualState::Success);
        graph
            .nodes
            .insert("Validating".to_string(), NodeVisualState::Active);
        graph
            .nodes
            .insert("Classifying".to_string(), NodeVisualState::Idle);
        graph.current_state = "Validating".to_string();
        graph.accumulated_confidence = 0.85;
        graph
            .edges
            .push(EdgeVisual::new("Ingested", "Validating", "start"));

        let svg = graph.to_animated_svg(None);
        assert!(svg.starts_with("<svg"), "SVG should start with svg tag");
        assert!(svg.contains("</svg>"), "SVG should have closing tag");
        assert!(svg.contains("Ingested"), "SVG should contain node name");
        assert!(svg.contains("Validating"), "SVG should contain node name");
        assert!(svg.contains("Classifying"), "SVG should contain node name");
        assert!(svg.contains("#4caf50"), "SVG should contain Success fill");
        assert!(svg.contains("#4a90d9"), "SVG should contain Active fill");
        assert!(svg.contains("line"), "SVG should contain edge lines");
        assert!(svg.contains("rect"), "SVG should contain node rects");
        assert!(
            !svg.contains("animateTransform"),
            "Static SVG should have no animation"
        );
    }

    #[test]
    fn test_animated_svg_reflow() {
        // graph1: narrow layout; current_state = Ingested (width ~120)
        let mut graph1 = PipelineGraph::new();
        graph1
            .nodes
            .insert("Ingested".to_string(), NodeVisualState::Active);
        graph1
            .nodes
            .insert("Validating".to_string(), NodeVisualState::Idle);
        graph1
            .nodes
            .insert("Classifying".to_string(), NodeVisualState::Idle);
        graph1.current_state = "Ingested".to_string();
        graph1.accumulated_confidence = 0.9;

        // graph2: different current_state => Ingested shrinks, Classifying expands => positions shift
        let mut graph2 = PipelineGraph::new();
        graph2
            .nodes
            .insert("Ingested".to_string(), NodeVisualState::Success);
        graph2
            .nodes
            .insert("Validating".to_string(), NodeVisualState::Success);
        graph2
            .nodes
            .insert("Classifying".to_string(), NodeVisualState::Active);
        graph2.current_state = "Classifying".to_string();
        graph2.accumulated_confidence = 0.9;

        let svg = graph2.to_animated_svg(Some(&graph1));
        assert!(svg.starts_with("<svg"), "SVG should start with svg tag");
        assert!(svg.contains("</svg>"), "SVG should have closing tag");
        assert!(
            svg.contains("animateTransform"),
            "Reflow SVG should have animateTransform"
        );
        assert!(
            svg.contains("from=\""),
            "animateTransform should have from attribute"
        );
        assert!(
            svg.contains("fill=\"freeze\""),
            "animateTransform should freeze"
        );

        // static same-graph call should have no animation
        let static_svg = graph2.to_animated_svg(None);
        assert!(
            !static_svg.contains("animateTransform"),
            "Static SVG should have no animation"
        );
    }
}
