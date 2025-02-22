use ankiview::application::NoteRepository;
use ankiview::infrastructure::AnkiRepository;
use anyhow::Result;
use tempfile::TempDir;

fn setup_test_repository() -> Result<(TempDir, AnkiRepository)> {
    let temp_dir = tempfile::tempdir()?;
    let collection_path = temp_dir.path().join("collection.anki2");

    // TODO: Create a proper test collection using anki-core
    std::fs::write(&collection_path, vec![0; 100])?;

    let repo = AnkiRepository::new(&collection_path)?;
    Ok((temp_dir, repo))
}

#[test]
#[ignore = "TODO: Not implemented"]
fn given_nonexistent_note_when_getting_note_then_returns_error() -> Result<()> {
    // Arrange
    let (_temp_dir, mut repo) = setup_test_repository()?;

    // Act
    let result = repo.get_note(999999);

    // Assert
    assert!(matches!(result, Err(_)));
    Ok(())
}
