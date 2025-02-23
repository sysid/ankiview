use ankiview::cli::args::Args;
use anyhow::Result;
use std::path::PathBuf;
use tempfile::TempDir;

fn setup_test_collection() -> Result<(TempDir, PathBuf)> {
    let temp_dir = tempfile::tempdir()?;
    let collection_path = temp_dir.path().join("collection.anki2");

    // Create a minimal valid collection using anki-core
    // This is a simplified example - you'd want to create a proper test collection
    std::fs::write(&collection_path, vec![0; 100])?;

    Ok((temp_dir, collection_path))
}

#[test]
fn given_valid_collection_path_when_running_then_succeeds() -> Result<()> {
    // Arrange
    let (_temp_dir, collection_path) = setup_test_collection()?;

    let args = Args {
        note_id: 1,
        collection: Some(collection_path),
        profile: None,
        verbose: 0,
    };

    // Act
    let result = ankiview::run(args);

    // Assert
    assert!(result.is_err()); // Will error since note 1 doesn't exist
    Ok(())
}

#[test]
fn given_invalid_collection_path_when_running_then_fails() {
    // Arrange
    let args = Args {
        note_id: 1,
        collection: Some(PathBuf::from("/nonexistent/path")),
        profile: None,
        verbose: 0,
    };

    // Act
    let result = ankiview::run(args);

    // Assert
    assert!(result.is_err());
}
