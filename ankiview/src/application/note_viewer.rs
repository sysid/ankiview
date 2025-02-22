// src/application/note_viewer.rs
use crate::domain::{DomainError, Note};
use anyhow::Result;

pub trait NoteRepository {
    fn get_note(&mut self, id: i64) -> Result<Note, DomainError>;
}

pub struct NoteViewer<R: NoteRepository> {
    repository: R,
}

impl<R: NoteRepository> NoteViewer<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn view_note(&mut self, note_id: i64) -> Result<Note, DomainError> {
        self.repository.get_note(note_id)
    }
}
