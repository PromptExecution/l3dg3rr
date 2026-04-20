# Introduction

`l3dg3rr` is a local-first personal financial document intelligence system focused on retroactive U.S. expat tax preparation from raw PDF statements.

## Architecture

The system implements a 6-layer visualization stack:

| Layer | Module | Description |
|-------|--------|-------------|
| 0 | [graph](./graph.md) | NodeData, EdgeData, pipeline node/edge vectors |
| 1 | [layout](./layout.md) | ForceLayout with Fruchterman-Reingold solver |
| 2 | [layout](./iso.md) | Isometric projection (iso_project) |
| 3 | [render](./render.md) | GraphRenderer for screen coordinates |
| 4 | [slint_viz](./slint_viz.md) | SlintGraphView with thread-safe layout |
| 5 | host-window.rs | GraphView Slint component |

## Theory

See [Theory of Operation](./theory.md) for the Novel Theory of Tool pattern and executable documentation.

## Source Documentation

This book is auto-generated from Rustdoc comments in the source code.

```bash
cargo doc --workspace --no-deps
mdbook build
```