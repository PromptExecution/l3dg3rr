# Force Layout

The layout module implements force-directed graph layout using a custom Fruchterman-Reingold algorithm.

## ForceLayout

```rust
pub struct ForceLayout {
    positions: HashMap<usize, Vec3>,
    velocities: HashMap<usize, Vec3>,
}
```

## Methods

### new()

Creates an empty force layout.

### for_pipeline()

Creates a force layout initialized with the standard pipeline nodes arranged in a circle.

### tick()

Runs one iteration of the force-directed layout algorithm, updating node positions based on:
- Repulsion between all nodes
- Attraction along edges
- Center gravity to keep the graph bounded

### position(node_idx: usize) -> Option&lt;Vec3&gt;

Returns the 3D position of a node by index.

### all_positions() -> &HashMap&lt;usize, Vec3&gt;

Returns all node positions.

## Usage

See [Graph](./graph.md) for node creation and [Iso](./iso.md) for projection details.

```rust
let mut layout = ForceLayout::for_pipeline();
for _ in 0..100 {
    layout.tick();
}
for (idx, pos) in layout.all_positions() {
    println!("Node {}: {:?}", idx, pos);
}
```