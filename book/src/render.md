# Renderer

The render module provides screen coordinate mapping for the visualization.

## GraphRenderer

```rust
pub struct GraphRenderer {
    pub width: u32,
    pub height: u32,
    pub scale: f32,
    pub origin: Vec2,
}
```

## Methods

### new(width: u32, height: u32)

Creates a renderer with the specified canvas dimensions.

### screen_position(x: f32, y: f32, z: f32) -> Vec2

Converts 3D coordinates to 2D screen position using isometric projection.

## Usage

```rust
let renderer = GraphRenderer::new(800, 600);
let screen = renderer.screen_position(100.0, 0.0, 50.0);
```