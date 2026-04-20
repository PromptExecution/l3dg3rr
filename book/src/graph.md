# Graph Data Model

The graph module provides the data structures for the pipeline visualization.

## NodeData

```rust
pub struct NodeData {
    pub label: String,
    pub color: [f32; 3],
    pub mass: f32,
}
```

- **label**: Human-readable name for the node (e.g., "Ingested", "Validating")
- **color**: RGB values as floats [0-1] for visualization
- **mass**: Relative size/weight for force-directed layout

## EdgeData

```rust
pub struct EdgeData {
    pub weight: f32,
}
```

- **weight**: Edge strength for layout calculations

## Functions

### create_pipeline_nodes

Returns the standard pipeline node definitions:
- Ingested
- Validating  
- Classifying
- Reconciling
- Committed

### create_pipeline_edges

Returns the standard pipeline connections:
- Ingested → Validating
- Validating → Classifying
- Classifying → Reconciling
- Reconciling → Committed