mod helpers;

use ankiview::application::NoteRepository;
use ankiview::domain::DomainError;
use anyhow::Result;
use helpers::{test_notes, TestCollection};

// Existing test (now un-ignored)
#[test]
fn given_nonexistent_note_when_getting_note_then_returns_error() -> Result<()> {
    // Arrange
    let test_collection = TestCollection::new()?;
    let mut repo = test_collection.open_repository()?;

    // Act
    let result = repo.get_note(test_notes::NONEXISTENT);

    // Assert
    assert!(result.is_err());
    match result.unwrap_err() {
        DomainError::NoteNotFound(id) => assert_eq!(id, test_notes::NONEXISTENT),
        _ => panic!("Expected NoteNotFound error"),
    }
    Ok(())
}

#[test]
fn given_dag_note_when_getting_note_then_returns_note_with_image() -> Result<()> {
    // Arrange
    let test_collection = TestCollection::new()?;
    let mut repo = test_collection.open_repository()?;

    // Act
    let note = repo.get_note(test_notes::DAG_NOTE)?;

    // Assert
    assert_eq!(note.id, test_notes::DAG_NOTE);
    assert!(note.front.contains("DAG"));
    assert!(note.back.contains("dag.png")); // Has image reference
    assert!(!note.model_name.is_empty()); // Has a model name
    Ok(())
}

#[test]
fn given_tree_note_when_getting_note_then_returns_note() -> Result<()> {
    // Arrange
    let test_collection = TestCollection::new()?;
    let mut repo = test_collection.open_repository()?;

    // Act
    let note = repo.get_note(test_notes::TREE)?;

    // Assert
    assert_eq!(note.id, test_notes::TREE);
    assert!(note.front.contains("Tree"));
    assert!(!note.back.is_empty());
    Ok(())
}

#[test]
fn given_star_schema_note_when_getting_note_then_returns_html_content() -> Result<()> {
    // Arrange
    let test_collection = TestCollection::new()?;
    let mut repo = test_collection.open_repository()?;

    // Act
    let note = repo.get_note(test_notes::STAR_SCHEMA)?;

    // Assert
    assert!(note.back.contains("<h3>")); // Has HTML heading
    assert!(note.back.contains("star-schema.png")); // Has image
    assert!(note.back.contains("Fact Table"));
    Ok(())
}

#[test]
fn given_f1_score_note_when_getting_note_then_returns_data_science_content() -> Result<()> {
    // Arrange
    let test_collection = TestCollection::new()?;
    let mut repo = test_collection.open_repository()?;

    // Act
    let note = repo.get_note(test_notes::F1_SCORE)?;

    // Assert
    assert_eq!(note.id, test_notes::F1_SCORE);
    assert!(note.front.contains("F1 score"));
    Ok(())
}

#[test]
fn given_existing_note_when_deleting_then_removes_note() -> Result<()> {
    // Arrange
    let test_collection = TestCollection::new()?;
    let mut repo = test_collection.open_repository()?;

    // Verify note exists first
    let _ = repo.get_note(test_notes::TREE)?;

    // Act
    let deleted_cards = repo.delete_note(test_notes::TREE)?;

    // Assert
    assert!(deleted_cards > 0);

    // Verify note is gone
    let result = repo.get_note(test_notes::TREE);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn given_nonexistent_note_when_deleting_then_returns_error() -> Result<()> {
    // Arrange
    let test_collection = TestCollection::new()?;
    let mut repo = test_collection.open_repository()?;

    // Act
    let result = repo.delete_note(test_notes::NONEXISTENT);

    // Assert
    assert!(result.is_err());
    match result.unwrap_err() {
        DomainError::NoteNotFound(id) => assert_eq!(id, test_notes::NONEXISTENT),
        _ => panic!("Expected NoteNotFound error"),
    }
    Ok(())
}

#[test]
fn given_repository_when_accessing_media_dir_then_returns_valid_path() -> Result<()> {
    // Arrange
    let test_collection = TestCollection::new()?;
    let repo = test_collection.open_repository()?;

    // Act
    let media_dir = repo.media_dir();

    // Assert
    assert!(media_dir.exists());
    assert!(media_dir.is_dir());
    assert!(media_dir.ends_with("collection.media"));
    Ok(())
}

#[test]
fn given_collection_when_listing_all_notes_then_returns_all_notes() -> Result<()> {
    // Arrange
    let test_collection = TestCollection::new()?;
    let mut repo = test_collection.open_repository()?;

    // Act
    let notes = repo.list_notes(None)?;

    // Assert
    assert!(notes.len() >= 10); // Test collection has at least 10 notes
    assert!(notes.iter().any(|n| n.id == test_notes::TREE));
    assert!(notes.iter().any(|n| n.id == test_notes::DAG_NOTE));
    Ok(())
}

#[test]
fn given_collection_when_listing_with_search_then_returns_filtered_notes() -> Result<()> {
    // Arrange
    let test_collection = TestCollection::new()?;
    let mut repo = test_collection.open_repository()?;

    // Act
    let notes = repo.list_notes(Some("Tree"))?;

    // Assert
    assert!(notes.len() > 0);
    assert!(notes.iter().any(|n| n.front.contains("Tree")));
    Ok(())
}

#[test]
fn given_collection_when_searching_nonexistent_term_then_returns_empty() -> Result<()> {
    // Arrange
    let test_collection = TestCollection::new()?;
    let mut repo = test_collection.open_repository()?;

    // Act
    let notes = repo.list_notes(Some("xyznonexistent"))?;

    // Assert
    assert_eq!(notes.len(), 0);
    Ok(())
}
