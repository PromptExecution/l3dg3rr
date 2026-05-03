use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeData {
    pub label: String,
    pub color: [f32; 3],
    pub mass: f32,
}

impl NodeData {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            color: [0.6, 0.7, 0.8],
            mass: 1.0,
        }
    }

    pub fn with_color(mut self, color: [f32; 3]) -> Self {
        self.color = color;
        self
    }

    pub fn with_mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeData {
    pub weight: f32,
}

impl EdgeData {
    pub fn new(weight: f32) -> Self {
        Self { weight }
    }
}

pub fn create_pipeline_nodes() -> Vec<NodeData> {
    vec![
        NodeData::new("Ingested"),
        NodeData::new("Validating"),
        NodeData::new("Classifying"),
        NodeData::new("Reconciling"),
        NodeData::new("Committed"),
    ]
}

pub fn create_pipeline_edges() -> Vec<(usize, usize)> {
    vec![(0, 1), (1, 2), (2, 3), (3, 4)]
}
