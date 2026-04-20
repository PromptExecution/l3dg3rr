# Isometric Projection

The isometric projection converts 3D force-directed positions to 2D screen coordinates.

## iso_project

```rust
pub fn iso_project(p: Vec3, scale: f32, origin: Vec2) -> Vec2
```

### Parameters

- **p**: 3D position from force layout (Vec3)
- **scale**: Pixel scale factor (default: 32.0)
- **origin**: Screen center point (Vec2)

### Returns

2D screen coordinates (Vec2)

### Formula

```rust
x = origin.x + (p.x - p.z) * scale * 0.866  // cos(30°)
y = origin.y + (p.x + p.z) * scale * 0.5 - p.y * scale
```

This produces a classic dimetric isometric view with 2:1 pixel ratio.

### Usage

```rust
let screen_pos = iso_project(node_position, 32.0, Vec2::new(400.0, 300.0));
```