// src/domain/note.rs
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Note {
    pub id: i64,
    pub front: String,
    pub back: String,
    pub tags: Vec<String>,
    pub model_name: String,
}
