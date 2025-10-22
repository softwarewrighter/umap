use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkRecord {
    pub id: i64,
    pub source: String,
    pub chunk_index: i64,
    pub text: String,
    pub dim: usize,
    // Not serialized by default to avoid huge payloads unless needed
    #[serde(skip_serializing)]
    pub vector: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point2D {
    pub id: i64,
    pub source: String,
    pub chunk_index: i64,
    pub score: f32,
    pub text_preview: String,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point3D {
    pub id: i64,
    pub source: String,
    pub chunk_index: i64,
    pub score: f32,
    pub text_preview: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
