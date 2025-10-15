mod helpers;

use ankiview::application::NoteViewer;
use ankiview::ports::HtmlPresenter;
use anyhow::Result;
use helpers::{TestCollection, test_notes};

#[test]
fn given_valid_note_id_when_viewing_note_then_returns_note() -> Result<()> {
    // Arrange
    let test_collection = TestCollection::new()?;
    let repo = test_collection.open_repository()?;
    let mut viewer = NoteViewer::new(repo);

    // Act
    let note = viewer.view_note(test_notes::TREE)?;

    // Assert
    assert_eq!(note.id, test_notes::TREE);
    assert!(!note.front.is_empty());
    assert!(!note.back.is_empty());
    Ok(())
}

#[test]
fn given_nonexistent_note_id_when_viewing_note_then_returns_error() -> Result<()> {
    // Arrange
    let test_collection = TestCollection::new()?;
    let repo = test_collection.open_repository()?;
    let mut viewer = NoteViewer::new(repo);

    // Act
    let result = viewer.view_note(test_notes::NONEXISTENT);

    // Assert
    assert!(result.is_err());
    Ok(())
}

#[test]
fn given_dag_note_when_viewing_and_rendering_then_produces_valid_html_with_image() -> Result<()> {
    // Arrange
    let test_collection = TestCollection::new()?;
    let repo = test_collection.open_repository()?;
    let media_dir = test_collection.media_dir.clone();

    let mut viewer = NoteViewer::new(repo);
    let presenter = HtmlPresenter::with_media_dir(&media_dir);

    // Act
    let note = viewer.view_note(test_notes::DAG_NOTE)?;
    let html = presenter.render(&note);

    // Assert
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("file://"));
    assert!(html.contains("dag.png"));
    Ok(())
}

#[test]
fn given_star_schema_note_when_viewing_and_rendering_then_resolves_media_paths() -> Result<()> {
    // Arrange
    let test_collection = TestCollection::new()?;
    let repo = test_collection.open_repository()?;
    let media_dir = test_collection.media_dir.clone();

    let mut viewer = NoteViewer::new(repo);
    let presenter = HtmlPresenter::with_media_dir(&media_dir);

    // Act
    let note = viewer.view_note(test_notes::STAR_SCHEMA)?;
    let html = presenter.render(&note);

    // Assert
    assert!(html.contains("file://"));
    assert!(html.contains("star-schema.png"));
    Ok(())
}
