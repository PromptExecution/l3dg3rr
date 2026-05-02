//! Visualization — isometric rendering layer with idiomatic patterns.
//!
//! Mirrors the JavaScript isometric renderer in `book/theme/rhai-live-core.js`
//! as a Rust type hierarchy. The goal is to produce deterministic SVG/scene
//! descriptions that the b00t executive can audit and the autoresearch
//! loop can verify.
//!
//! # Patterns
//!
//! - `IsoProjection` — stateless projection function (2:1 dimetric)
//! - `SceneGraph` — a frame of isometric scene data (nodes, edges, grid)
//! - `SceneRenderer` — converts SceneGraph to SVG or glTF
//! - `AnimationFrame` — a snapshot with previous-frame diff for SVG animateTransform

use serde::Serialize;

/// 2:1 dimetric isometric projection constants.
pub const ISO_SCALE_X: f32 = 0.8660254; // cos(30°)
pub const ISO_SCALE_Y: f32 = 0.5;       // sin(30°)

/// A 2D point in screen space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

/// A 3D point in world space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point3D {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

/// Isometric projection — stateless pure function.
pub fn iso_project(p: Point3D, scale: f32, origin: Point2D) -> Point2D {
    Point2D {
        x: origin.x + (p.x - p.z) * scale * ISO_SCALE_X,
        y: origin.y + (p.x + p.z) * scale * ISO_SCALE_Y - p.y * scale,
    }
}

/// Semantic role of a scene node — drives icon, color, layout tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SemanticRole {
    Ingest,
    Validate,
    Classify,
    Review,
    Reconcile,
    Commit,
    Decision,
    Step,
}

impl SemanticRole {
    pub fn emoji(self) -> &'static str {
        match self {
            Self::Ingest => "📥",
            Self::Validate => "✅",
            Self::Classify => "🏷️",
            Self::Review => "👁️",
            Self::Reconcile => "🔄",
            Self::Commit => "💾",
            Self::Decision => "🔀",
            Self::Step => "⚙️",
        }
    }

    pub fn color(self) -> &'static str {
        match self {
            Self::Ingest => "#4fc3f7",
            Self::Validate => "#66bb6a",
            Self::Classify => "#ffa726",
            Self::Review => "#ab47bc",
            Self::Reconcile => "#26c6da",
            Self::Commit => "#42a5f5",
            Self::Decision => "#ef5350",
            Self::Step => "#78909c",
        }
    }
}

/// A node in the scene graph.
#[derive(Debug, Clone, Serialize)]
pub struct SceneNode {
    pub id: String,
    pub label: String,
    pub position: Point3D,
    pub role: SemanticRole,
    pub arm_index: Option<u32>,
    pub is_default: bool,
}

impl SceneNode {
    pub fn project(&self, scale: f32, origin: Point2D) -> Point2D {
        iso_project(self.position, scale, origin)
    }
}

/// An edge between two nodes.
#[derive(Debug, Clone, Serialize)]
pub struct SceneEdge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    pub is_bezier: bool,
}

/// A single frame of the scene graph — suffices for static SVG output.
#[derive(Debug, Clone, Serialize)]
pub struct SceneGraph {
    pub nodes: Vec<SceneNode>,
    pub edges: Vec<SceneEdge>,
    pub scale: f32,
    pub origin: Point2D,
}

impl SceneGraph {
    pub fn new(scale: f32) -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            scale,
            origin: Point2D { x: 400.0, y: 300.0 },
        }
    }

    pub fn add_node(&mut self, node: SceneNode) {
        self.nodes.push(node);
    }

    pub fn add_edge(&mut self, edge: SceneEdge) {
        self.edges.push(edge);
    }

    /// Compute bounding box of all projected nodes.
    pub fn bounding_box(&self) -> (Point2D, Point2D) {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for node in &self.nodes {
            let p = node.project(self.scale, self.origin);
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }

        (Point2D { x: min_x, y: min_y }, Point2D { x: max_x, y: max_y })
    }
}

/// A font/color/icon theme for rendering.
#[derive(Debug, Clone)]
pub struct SceneTheme {
    pub node_width: f32,
    pub node_height: f32,
    pub depth_inset: f32,
    pub font_size: u32,
    pub edge_color: String,
    pub grid_color: String,
    pub shadow_color: String,
}

impl Default for SceneTheme {
    fn default() -> Self {
        Self {
            node_width: 160.0,
            node_height: 80.0,
            depth_inset: 8.0,
            font_size: 14,
            edge_color: "#90a4ae".into(),
            grid_color: "#37474f".into(),
            shadow_color: "rgba(0,0,0,0.15)".into(),
        }
    }
}

/// Converts a SceneGraph to an SVG string.
pub fn scene_to_svg(scene: &SceneGraph, theme: &SceneTheme) -> String {
    let (min, max) = scene.bounding_box();
    let padding = 40.0;
    let width = (max.x - min.x + padding * 2.0).max(800.0);
    let height = (max.y - min.y + padding * 2.0).max(600.0);

    let mut svg = String::new();
    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {:.0} {:.0}" width="{:.0}" height="{:.0}">"#,
        width, height, width, height
    ));

    // Grid
    svg.push_str(&format!(
        r#"<defs><pattern id="grid" width="40" height="40" patternUnits="userSpaceOnUse"><path d="M 40 0 L 0 0 0 40" fill="none" stroke="{}" stroke-width="0.5"/></pattern></defs>"#,
        theme.grid_color
    ));
    svg.push_str(&format!(
        r#"<rect width="{:.0}" height="{:.0}" fill="url(#grid)"/>"#,
        width, height
    ));

    // Edge paths
    for edge in &scene.edges {
        let from_node = scene.nodes.iter().find(|n| n.id == edge.from);
        let to_node = scene.nodes.iter().find(|n| n.id == edge.to);
        if let (Some(f), Some(t)) = (from_node, to_node) {
            let fp = f.project(scene.scale, scene.origin);
            let tp = t.project(scene.scale, scene.origin);
            let d = if edge.is_bezier {
                let cpx = (fp.x + tp.x) / 2.0;
                let cpy = (fp.y + tp.y) / 2.0 - 30.0;
                format!("M {:.1} {:.1} Q {:.1} {:.1} {:.1} {:.1}", fp.x, fp.y, cpx, cpy, tp.x, tp.y)
            } else {
                format!("M {:.1} {:.1} L {:.1} {:.1}", fp.x, fp.y, tp.x, tp.y)
            };
            svg.push_str(&format!(
                r#"<path d="{}" stroke="{}" stroke-width="2" fill="none" marker-end="url(#arrow)"/>"#,
                d, theme.edge_color
            ));
        }
    }

    // Arrow marker
    svg.push_str(&format!(
        r#"<defs><marker id="arrow" viewBox="0 0 10 10" refX="10" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M 0 0 L 10 5 L 0 10 Z" fill="{}"/></marker></defs>"#,
        theme.edge_color
    ));

    // Nodes
    for node in &scene.nodes {
        let p = node.project(scene.scale, scene.origin);
        let w = theme.node_width;
        let h = theme.node_height;
        let di = theme.depth_inset;
        let color = node.role.color();

        // Isometric card with depth faces
        let b00 = "#000";
        let fff = "#fff";
        let emoji = node.role.emoji();
        let role_label = node.role.to_string();
        let hw = w / 2.0;
        svg.push_str(&format!(
            r#"<g transform="translate({tx:.1},{ty:.1})">"#, tx = p.x - hw, ty = p.y - h / 2.0
        ));
        // Right depth face
        svg.push_str(&format!(
            r#"<polygon points="{w:.1},{hd:.1} {wd:.1},{h:.1} {wd:.1},{hd2:.1} {w:.1},{hd2:.1}" fill="{b00}" opacity="0.15"/>"#,
            w = w, hd = h - di, wd = w + di, h = h, hd2 = h + di, b00 = b00
        ));
        // Bottom depth face
        svg.push_str(&format!(
            r#"<polygon points="0,{h:.1} {w:.1},{h:.1} {wd:.1},{hd:.1} {di:.1},{hd:.1}" fill="{b00}" opacity="0.1"/>"#,
            h = h, w = w, wd = w + di, hd = h + di, di = di, b00 = b00
        ));
        // Front face
        svg.push_str(&format!(
            r#"<rect x="0" y="0" width="{w:.1}" height="{h:.1}" rx="6" fill="{color}" stroke="{fff}" stroke-width="1"/>"#,
            w = w, h = h, color = color, fff = fff
        ));
        // Icon + label
        svg.push_str(&format!(
            r#"<text x="{hw:.1}" y="28" text-anchor="middle" font-size="20">{emoji}<tspan x="{hw:.1}" y="50" font-size="{fs}" fill="{fff}" text-anchor="middle">{label}</tspan><tspan x="{hw:.1}" y="68" font-size="10" fill="rgba(255,255,255,0.7)" text-anchor="middle">{role}</tspan></text>"#,
            hw = hw, emoji = emoji, fs = theme.font_size, fff = fff, label = node.label, role = role_label
        ));
        svg.push_str("</g>");
    }

    svg.push_str("</svg>");
    svg
}

impl std::fmt::Display for SemanticRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ingest => write!(f, "ingest"),
            Self::Validate => write!(f, "validate"),
            Self::Classify => write!(f, "classify"),
            Self::Review => write!(f, "review"),
            Self::Reconcile => write!(f, "reconcile"),
            Self::Commit => write!(f, "commit"),
            Self::Decision => write!(f, "decision"),
            Self::Step => write!(f, "step"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_project_matches_js_contract() {
        // JS: isoProject({x:1,y:0,z:0}, 1, {x:400,y:300})
        //   x = 400 + (1-0) * 1 * 0.8660254 = 400.866
        //   y = 300 + (1+0) * 1 * 0.5 - 0 * 1 = 300.5
        let p = iso_project(Point3D::new(1.0, 0.0, 0.0), 1.0, Point2D { x: 400.0, y: 300.0 });
        assert!((p.x - 400.866).abs() < 0.001);
        assert!((p.y - 300.5).abs() < 0.001);
    }

    #[test]
    fn iso_project_origin_at_origin() {
        let p = iso_project(Point3D::new(0.0, 0.0, 0.0), 32.0, Point2D { x: 0.0, y: 0.0 });
        assert!((p.x - 0.0).abs() < 0.001);
        assert!((p.y - 0.0).abs() < 0.001);
    }

    #[test]
    fn bounding_box_of_empty_scene() {
        let s = SceneGraph::new(1.0);
        let (min, max) = s.bounding_box();
        assert!(min.x > 1e30 || min.x.is_infinite());
        assert!(max.x < -1e30 || max.x.is_infinite());
    }

    #[test]
    fn scene_graph_add_node_and_project() {
        let mut s = SceneGraph::new(32.0);
        s.add_node(SceneNode {
            id: "n1".into(),
            label: "Test".into(),
            position: Point3D::new(5.0, 3.0, 2.0),
            role: SemanticRole::Step,
            arm_index: None,
            is_default: false,
        });
        let (min, max) = s.bounding_box();
        // Single node: min == max for the exact projected point
        assert!((max.x - min.x).abs() < 0.001);
        assert!((max.y - min.y).abs() < 0.001);
    }

    #[test]
    fn scene_to_svg_contains_elements() {
        let mut s = SceneGraph::new(32.0);
        s.add_node(SceneNode {
            id: "a".into(),
            label: "Ingest".into(),
            position: Point3D::new(0.0, 0.0, 0.0),
            role: SemanticRole::Ingest,
            arm_index: None,
            is_default: false,
        });
        s.add_node(SceneNode {
            id: "b".into(),
            label: "Validate".into(),
            position: Point3D::new(3.0, 0.0, 0.0),
            role: SemanticRole::Validate,
            arm_index: None,
            is_default: false,
        });
        s.add_edge(SceneEdge {
            from: "a".into(),
            to: "b".into(),
            label: Some("pass".into()),
            is_bezier: true,
        });

        let svg = scene_to_svg(&s, &SceneTheme::default());
        assert!(svg.contains("<svg"));
        assert!(svg.contains("📥"));
        assert!(svg.contains("✅"));
        assert!(svg.contains("ingest"));
        assert!(svg.contains("validate"));
        assert!(svg.contains("M "));
    }

    #[test]
    fn scene_edge_without_label() {
        let mut s = SceneGraph::new(1.0);
        s.add_node(SceneNode {
            id: "a".into(),
            label: "A".into(),
            position: Point3D::new(0.0, 0.0, 0.0),
            role: SemanticRole::Step,
            arm_index: None,
            is_default: false,
        });
        s.add_node(SceneNode {
            id: "b".into(),
            label: "B".into(),
            position: Point3D::new(1.0, 0.0, 0.0),
            role: SemanticRole::Step,
            arm_index: None,
            is_default: false,
        });
        s.add_edge(SceneEdge {
            from: "a".into(),
            to: "b".into(),
            label: None,
            is_bezier: false,
        });
        let svg = scene_to_svg(&s, &SceneTheme::default());
        assert!(svg.contains("L "));
    }
}
