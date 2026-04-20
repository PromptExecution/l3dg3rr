# Visualization

The visualize module generates Mermaid diagrams and HTML exports for pipeline state.

## PipelineGraph

```rust
pub struct PipelineGraph {
    pub nodes: HashMap<String, NodeVisualState>,
    pub edges: Vec<EdgeVisual>,
    pub current_state: String,
    pub accumulated_confidence: f32,
}
```

## NodeVisualState

Visual states for pipeline nodes:
- **Idle**: Waiting, not entered
- **Active**: Currently executing
- **Success**: Completed successfully
- **Warning**: Recoverable issue
- **Error**: Unrecoverable issue
- **Review**: Awaiting human review

## Mermaid Generation

```rust
let mermaid = graph.to_mermaid();
// Generates stateDiagram-v2
```

## HTML Export

```rust
let html = to_html(&graph);
// Returns complete HTML with Mermaid.js
```

## LayoutSolver

Kasuari-backed constraint solver for node positioning:

```rust
pub struct LayoutSolver;
pub fn generate_layout(&self, graph: &PipelineGraph) -> HashMap<String, (f32, f32)>;
```