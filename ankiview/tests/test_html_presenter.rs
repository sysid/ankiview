mod helpers;

use ankiview::application::NoteRepository;
use ankiview::ports::HtmlPresenter;
use anyhow::Result;
use helpers::{TestCollection, test_notes};

#[test]
fn given_dag_note_when_rendering_with_media_dir_then_converts_to_file_uri() -> Result<()> {
    // Arrange
    let test_collection = TestCollection::new()?;
    let mut repo = test_collection.open_repository()?;
    let note = repo.get_note(test_notes::DAG_NOTE)?;

    let media_dir = test_collection.media_dir.clone();
    let presenter = HtmlPresenter::with_media_dir(&media_dir);

    // Act
    let html = presenter.render(&note);

    // Assert
    assert!(html.contains("file://"));
    assert!(html.contains("dag.png"));
    assert!(html.contains(&media_dir.to_string_lossy().to_string()));
    Ok(())
}

#[test]
fn given_star_schema_note_when_rendering_then_processes_html_content() -> Result<()> {
    // Arrange
    let test_collection = TestCollection::new()?;
    let mut repo = test_collection.open_repository()?;
    let note = repo.get_note(test_notes::STAR_SCHEMA)?;

    let media_dir = test_collection.media_dir.clone();
    let presenter = HtmlPresenter::with_media_dir(&media_dir);

    // Act
    let html = presenter.render(&note);

    // Assert
    assert!(html.contains("file://"));  // Image converted to file URI
    assert!(html.contains("star-schema.png"));
    assert!(html.contains("<h3>"));  // HTML structure preserved
    assert!(html.contains("Fact Table"));
    Ok(())
}

#[test]
fn given_mercator_note_when_rendering_then_converts_multiple_images() -> Result<()> {
    // Arrange
    let test_collection = TestCollection::new()?;
    let mut repo = test_collection.open_repository()?;
    let note = repo.get_note(test_notes::MERCATOR)?;

    let media_dir = test_collection.media_dir.clone();
    let presenter = HtmlPresenter::with_media_dir(&media_dir);

    // Act
    let html = presenter.render(&note);

    // Assert
    assert!(html.contains("mercator.png"));
    assert!(html.contains("wsg-enu2.png"));
    assert!(html.contains("file://"));
    Ok(())
}

#[test]
fn given_f1_score_note_when_rendering_then_includes_syntax_highlighting() -> Result<()> {
    // Arrange
    let test_collection = TestCollection::new()?;
    let mut repo = test_collection.open_repository()?;
    let note = repo.get_note(test_notes::F1_SCORE)?;

    let presenter = HtmlPresenter::new();

    // Act
    let html = presenter.render(&note);

    // Assert
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("F1 score"));
    // highlight.js should be included for potential code blocks
    assert!(html.contains("highlight.js"));
    Ok(())
}

#[test]
fn given_recursive_dfs_note_when_rendering_then_handles_code_content() -> Result<()> {
    // Arrange
    let test_collection = TestCollection::new()?;
    let mut repo = test_collection.open_repository()?;
    let note = repo.get_note(test_notes::RECURSIVE_DFS)?;

    let presenter = HtmlPresenter::new();

    // Act
    let html = presenter.render(&note);

    // Assert - should not crash, should have valid HTML structure
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("DFS") || html.contains("recursive"));
    Ok(())
}
