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

    /// List all available note types (models) in the collection
    /// Returns a vector of (notetype_id, notetype_name) tuples
    fn list_notetypes(&mut self) -> Result<Vec<(i64, String)>, DomainError>;

    /// Add tags to an existing note (merge: existing tags preserved)
    fn add_tags(&mut self, id: i64, tags: &[String]) -> Result<(), DomainError>;

    /// Remove specific tags from an existing note
    fn remove_tags(&mut self, id: i64, tags: &[String]) -> Result<(), DomainError>;

    /// Update both fields and tags on an existing note
    fn update_note_fields_and_tags(
        &mut self,
        id: i64,
        fields: &[String],
        tags: &[String],
    ) -> Result<(), DomainError>;

    /// Replace a tag across notes matching an optional query.
    /// If old_tag is empty: adds new_tag to all matching notes (bulk add).
    /// If new_tag is empty: removes old_tag from all matching notes (bulk remove).
    /// Returns the number of notes affected.
    fn replace_tag(
        &mut self,
        query: Option<&str>,
        old_tag: &str,
        new_tag: &str,
    ) -> Result<usize, DomainError>;
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
