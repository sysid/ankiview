mod helpers;

use ankiview::application::NoteViewer;
use ankiview::ports::HtmlPresenter;
use anyhow::Result;
use helpers::{test_notes, TestCollection};

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

#[test]
fn given_valid_note_when_viewing_as_json_then_returns_valid_json_string() -> Result<()> {
    // Arrange
    let test_collection = TestCollection::new()?;
    let repo = test_collection.open_repository()?;
    let mut viewer = NoteViewer::new(repo);

    // Act
    let note = viewer.view_note(test_notes::TREE)?;
    let json = serde_json::to_string_pretty(&note)?;

    // Assert
    assert!(json.contains(r#""id":"#));
    assert!(json.contains(&test_notes::TREE.to_string()));
    assert!(json.contains(r#""front":"#));
    assert!(json.contains(r#""back":"#));
    assert!(json.contains(r#""tags":"#));
    assert!(json.contains(r#""model_name":"#));

    // Verify it's valid JSON by parsing it back
    let parsed: serde_json::Value = serde_json::from_str(&json)?;
    assert_eq!(parsed["id"].as_i64().unwrap(), test_notes::TREE);

    Ok(())
}

#[test]
fn given_note_with_html_content_when_serializing_to_json_then_preserves_html() -> Result<()> {
    // Arrange
    let test_collection = TestCollection::new()?;
    let repo = test_collection.open_repository()?;
    let mut viewer = NoteViewer::new(repo);

    // Act
    let note = viewer.view_note(test_notes::DAG_NOTE)?;
    let json = serde_json::to_string_pretty(&note)?;

    // Assert - HTML tags should be preserved as strings
    assert!(json.contains(r#""front":"#));
    // Note: The actual HTML content will be escaped in JSON strings
    let parsed: serde_json::Value = serde_json::from_str(&json)?;
    assert!(!parsed["front"].as_str().unwrap().is_empty());

    Ok(())
}
