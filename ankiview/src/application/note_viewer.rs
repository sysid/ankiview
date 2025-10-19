// src/application/note_viewer.rs
use crate::domain::{DomainError, Note};
use anyhow::Result;

pub trait NoteRepository {
    fn get_note(&mut self, id: i64) -> Result<Note, DomainError>;

    /// Delete a note and all associated cards from the collection
    /// Returns the number of cards deleted
    fn delete_note(&mut self, id: i64) -> Result<usize, DomainError>;

    /// List notes, optionally filtered by a search query.
    /// If search_query is None, returns all notes.
    /// If search_query is Some(query), returns notes matching the query.
    fn list_notes(&mut self, search_query: Option<&str>) -> Result<Vec<Note>, DomainError>;
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
