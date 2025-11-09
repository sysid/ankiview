// src/application/note_lister.rs
use crate::application::NoteRepository;
use crate::domain::{DomainError, Note};

pub struct NoteLister<R: NoteRepository> {
    repository: R,
}

impl<R: NoteRepository> NoteLister<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// List all notes, or filter by search query
    ///
    /// # Arguments
    /// * `search_query` - Optional search term to filter front field
    ///
    /// # Returns
    /// Vector of notes matching the criteria
    pub fn list_notes(&mut self, search_query: Option<&str>) -> Result<Vec<Note>, DomainError> {
        self.repository.list_notes(search_query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Note;
    use crate::util::testing::MockNoteRepository;

    #[test]
    fn given_no_search_when_listing_notes_then_returns_all_notes() {
        // Arrange
        let note1 = Note {
            id: 1,
            front: "First".to_string(),
            back: "Back1".to_string(),
            tags: vec![],
            model_name: "Basic".to_string(),
        };
        let note2 = Note {
            id: 2,
            front: "Second".to_string(),
            back: "Back2".to_string(),
            tags: vec![],
            model_name: "Basic".to_string(),
        };

        let mock = MockNoteRepository::builder()
            .with_note(1, note1)
            .with_note(2, note2)
            .build();
        let mut lister = NoteLister::new(mock);

        // Act
        let result = lister
            .list_notes(None)
            .expect("List should succeed");

        // Assert
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn given_search_query_when_listing_notes_then_returns_filtered_notes() {
        // Arrange
        let note1 = Note {
            id: 1,
            front: "What is a Tree?".to_string(),
            back: "Back1".to_string(),
            tags: vec![],
            model_name: "Basic".to_string(),
        };
        let note2 = Note {
            id: 2,
            front: "What is a Graph?".to_string(),
            back: "Back2".to_string(),
            tags: vec![],
            model_name: "Basic".to_string(),
        };

        let mock = MockNoteRepository::builder()
            .with_note(1, note1)
            .with_note(2, note2)
            .build();
        let mut lister = NoteLister::new(mock);

        // Act
        let result = lister
            .list_notes(Some("Tree"))
            .expect("List should succeed");

        // Assert
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, 1);
    }
}
