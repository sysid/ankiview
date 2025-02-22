// src/domain/note.rs
#[derive(Debug, Clone)]
pub struct Note {
    pub id: i64,
    pub front: String,
    pub back: String,
    pub tags: Vec<String>,
    pub model_name: String,
}
