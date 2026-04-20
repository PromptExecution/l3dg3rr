# Slint Visualization

The slint_viz module integrates the graph visualization with Slint UI framework.

## SlintGraphView

```rust
pub struct SlintGraphView {
    pub renderer: GraphRenderer,
    pub layout: Arc<RwLock<ForceLayout>>,
}
```

## Features

- Thread-safe layout using `Arc<RwLock<ForceLayout>>`
- Animation support via timer-driven tick updates
- Real-time position updates for Slint binding

## Methods

### new(width: u32, height: u32)

Creates a new Slint-integrated graph view.

### tick()

Advances the force layout by one iteration. Thread-safe.

### screen_position(node_idx: usize) -> Option<(f32, f32)>

Returns the screen position for a node.

## Integration

```rust
let view = SlintGraphView::new(800, 600);

// In animation loop:
view.tick();
let pos = view.screen_position(0);
```