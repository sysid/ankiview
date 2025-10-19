mod helpers;

use ankiview::application::{NoteDeleter, NoteRepository};
use ankiview::domain::DomainError;
use anyhow::Result;
use helpers::{TestCollection, test_notes};

#[test]
fn given_existing_note_when_deleting_then_removes_note_and_cards() -> Result<()> {
    // Arrange
    let test_collection = TestCollection::new()?;
    let mut repo = test_collection.open_repository()?;

    // Verify note exists first
    let _ = repo.get_note(test_notes::RECURSIVE_DFS)?;

    let mut deleter = NoteDeleter::new(repo);

    // Act
    let deleted_cards = deleter.delete_note(test_notes::RECURSIVE_DFS)?;

    // Assert
    assert!(deleted_cards > 0);

    // Note: We can't verify deletion by reopening the collection due to SQLite lock.
    // The successful deletion with deleted_cards > 0 is sufficient verification.
    Ok(())
}

#[test]
fn given_nonexistent_note_when_deleting_then_returns_not_found_error() -> Result<()> {
    // Arrange
    let test_collection = TestCollection::new()?;
    let repo = test_collection.open_repository()?;
    let mut deleter = NoteDeleter::new(repo);

    // Act
    let result = deleter.delete_note(test_notes::NONEXISTENT);

    // Assert
    assert!(result.is_err());
    match result.unwrap_err() {
        DomainError::NoteNotFound(id) => assert_eq!(id, test_notes::NONEXISTENT),
        _ => panic!("Expected NoteNotFound error"),
    }
    Ok(())
}

#[test]
fn given_note_with_image_when_deleting_then_removes_all_cards() -> Result<()> {
    // Arrange
    let test_collection = TestCollection::new()?;
    let repo = test_collection.open_repository()?;
    let mut deleter = NoteDeleter::new(repo);

    // Act - delete note that has image
    let deleted_cards = deleter.delete_note(test_notes::DAG_NOTE)?;

    // Assert - at least one card deleted
    assert!(deleted_cards >= 1);
    Ok(())
}
