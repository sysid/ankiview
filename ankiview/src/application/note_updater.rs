// src/application/note_updater.rs
use crate::application::NoteRepository;
use crate::domain::DomainError;

pub struct NoteUpdater<R: NoteRepository> {
    repository: R,
}

impl<R: NoteRepository> NoteUpdater<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn add_tags(&mut self, note_id: i64, tags: &[String]) -> Result<(), DomainError> {
        // Verify note exists first for a clear error
        self.repository.get_note(note_id)?;
        self.repository.add_tags(note_id, tags)
    }

    pub fn remove_tags(&mut self, note_id: i64, tags: &[String]) -> Result<(), DomainError> {
        self.repository.get_note(note_id)?;
        self.repository.remove_tags(note_id, tags)
    }
}
