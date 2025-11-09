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
    use crate::domain::DomainError;
    use crate::util::testing::MockNoteRepository;

    #[test]
    fn given_existing_note_when_deleting_then_returns_card_count() {
        // Arrange
        let mock = MockNoteRepository::builder()
            .with_delete_success(123, 3)
            .build();
        let mut deleter = NoteDeleter::new(mock);

        // Act
        let result = deleter.delete_note(123);

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.expect("Delete should succeed"), 3);
    }

    #[test]
    fn given_nonexistent_note_when_deleting_then_returns_error() {
        // Arrange
        let mock = MockNoteRepository::builder()
            .with_delete_not_found(999)
            .build();
        let mut deleter = NoteDeleter::new(mock);

        // Act
        let result = deleter.delete_note(999);

        // Assert
        assert!(result.is_err());
        match result.expect_err("Should return error") {
            DomainError::NoteNotFound(id) => assert_eq!(id, 999),
            _ => panic!("Expected NoteNotFound error"),
        }
    }
}
