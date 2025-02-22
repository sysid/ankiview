// src/domain/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Note not found: {0}")]
    NoteNotFound(i64),
    #[error("Profile error: {0}")]
    ProfileError(String),
    #[error("Collection error: {0}")]
    CollectionError(String),
}

