// src/application/note_deleter.rs
use crate::application::NoteRepository;
use crate::domain::DomainError;

pub struct NoteDeleter<R: NoteRepository> {
    repository: R,
}

impl<R: NoteRepository> NoteDeleter<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Delete a note and return the number of cards that were deleted
    pub fn delete_note(&mut self, note_id: i64) -> Result<usize, DomainError> {
        self.repository.delete_note(note_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DomainError, Note};
    use anyhow::Result;

    // Mock repository for testing
    struct MockRepository {
        should_succeed: bool,
        deleted_cards: usize,
    }

    impl NoteRepository for MockRepository {
        fn get_note(&mut self, _id: i64) -> Result<Note, DomainError> {
            unimplemented!("Not needed for deleter tests")
        }

        fn delete_note(&mut self, id: i64) -> Result<usize, DomainError> {
            if self.should_succeed {
                Ok(self.deleted_cards)
            } else {
                Err(DomainError::NoteNotFound(id))
            }
        }

        fn list_notes(&mut self, _search_query: Option<&str>) -> Result<Vec<Note>, DomainError> {
            unimplemented!("Not needed for deleter tests")
        }
    }

    #[test]
    fn given_existing_note_when_deleting_then_returns_card_count() {
        // Arrange
        let repo = MockRepository {
            should_succeed: true,
            deleted_cards: 3,
        };
        let mut deleter = NoteDeleter::new(repo);

        // Act
        let result = deleter.delete_note(123);

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
    }

    #[test]
    fn given_nonexistent_note_when_deleting_then_returns_error() {
        // Arrange
        let repo = MockRepository {
            should_succeed: false,
            deleted_cards: 0,
        };
        let mut deleter = NoteDeleter::new(repo);

        // Act
        let result = deleter.delete_note(999);

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::NoteNotFound(id) => assert_eq!(id, 999),
            _ => panic!("Expected NoteNotFound error"),
        }
    }
}
