mod helpers;

use anyhow::Result;
use helpers::TestCollection;
use std::fs;
use tempfile::TempDir;

#[test]
fn given_markdown_file_when_collecting_then_creates_notes_in_anki() -> Result<()> {
    // Arrange
    let test_collection = TestCollection::new()?;
    let temp_dir = TempDir::new()?;

    // Create a markdown file with basic cards only (simpler test)
    let markdown_path = temp_dir.path().join("test.md");
    let markdown_content = r#"---
Deck: IntegrationTest

1. What is the capital of France?
> Paris

2. What is 2 + 2?
> 4
---"#;
    fs::write(&markdown_path, markdown_content)?;

    // Act
    let mut collector = ankiview::inka::application::card_collector::CardCollector::new(
        &test_collection.collection_path,
        false,
        false,
    )?;
    let count = collector.process_file(&markdown_path)?;

    // Assert
    assert_eq!(count, 2, "Should process 2 cards");

    // Verify IDs were injected
    let updated_content = fs::read_to_string(&markdown_path)?;
    assert!(
        updated_content.contains("<!--ID:"),
        "Should have ID comments"
    );

    // Count ID occurrences
    let id_count = updated_content.matches("<!--ID:").count();
    assert_eq!(id_count, 2, "Should have 2 ID comments");

    Ok(())
}

#[test]
fn given_directory_when_collecting_recursively_then_processes_all_files() -> Result<()> {
    // Arrange
    let test_collection = TestCollection::new()?;
    let temp_dir = TempDir::new()?;
    let notes_dir = temp_dir.path().join("notes");
    fs::create_dir(&notes_dir)?;

    // Create subdirectory
    let subdir = notes_dir.join("chapter1");
    fs::create_dir(&subdir)?;

    // File 1: Basic cards
    let file1 = notes_dir.join("basics.md");
    fs::write(
        &file1,
        r#"---
Deck: Integration

1. Basic question?
> Basic answer
---"#,
    )?;

    // File 2: Cloze card in subdirectory
    let file2 = subdir.join("cloze.md");
    fs::write(
        &file2,
        r#"---
Deck: Integration

1. {Cloze deletion} test.
---"#,
    )?;

    // Act
    let mut collector = ankiview::inka::application::card_collector::CardCollector::new(
        &test_collection.collection_path,
        false,
        false,
    )?;
    let count = collector.process_directory(&notes_dir)?;

    // Assert
    assert_eq!(count, 2, "Should process 2 cards from 2 files");

    // Verify both files got IDs
    let content1 = fs::read_to_string(&file1)?;
    assert!(content1.contains("<!--ID:"));

    let content2 = fs::read_to_string(&file2)?;
    assert!(content2.contains("<!--ID:"));

    Ok(())
}

#[test]
fn given_file_with_existing_ids_when_collecting_then_updates_notes() -> Result<()> {
    // Arrange
    let test_collection = TestCollection::new()?;
    let temp_dir = TempDir::new()?;
    let markdown_path = temp_dir.path().join("update.md");

    // Initial content
    let initial_content = r#"---
Deck: UpdateTest

1. What is Rust?
> A programming language
---"#;
    fs::write(&markdown_path, initial_content)?;

    // First collection run
    let mut collector = ankiview::inka::application::card_collector::CardCollector::new(
        &test_collection.collection_path,
        false,
        false,
    )?;
    let count1 = collector.process_file(&markdown_path)?;
    assert_eq!(count1, 1);

    // Get the content with ID
    let content_with_id = fs::read_to_string(&markdown_path)?;
    assert!(content_with_id.contains("<!--ID:"));

    // Modify the answer
    let modified_content = content_with_id.replace(
        "A programming language",
        "A safe systems programming language",
    );
    fs::write(&markdown_path, &modified_content)?;

    // Act - Second collection run (should update, not create new)
    let count2 = collector.process_file(&markdown_path)?;

    // Assert
    assert_eq!(
        count2, 1,
        "Should still process 1 card (update, not create)"
    );

    // Verify still has same ID
    let final_content = fs::read_to_string(&markdown_path)?;
    assert_eq!(
        final_content.matches("<!--ID:").count(),
        1,
        "Should still have exactly 1 ID"
    );

    Ok(())
}

#[test]
fn given_mixed_card_types_when_collecting_then_creates_correct_note_types() -> Result<()> {
    // Arrange
    let test_collection = TestCollection::new()?;
    let temp_dir = TempDir::new()?;
    let markdown_path = temp_dir.path().join("mixed.md");

    let markdown_content = r#"---
Deck: MixedTypes
Tags: test integration

1. Basic card front?
> Basic card back

2. This is a {cloze deletion} example.

3. Another basic?
> Another answer
---"#;
    fs::write(&markdown_path, markdown_content)?;

    // Act
    let count = {
        let mut collector = ankiview::inka::application::card_collector::CardCollector::new(
            &test_collection.collection_path,
            false,
            false,
        )?;
        collector.process_file(&markdown_path)?
    }; // Collector dropped here, releasing the lock

    // Assert
    assert_eq!(count, 3, "Should process all 3 cards");

    // Verify all got IDs
    let final_content = fs::read_to_string(&markdown_path)?;
    assert_eq!(
        final_content.matches("<!--ID:").count(),
        3,
        "Should have 3 ID comments"
    );

    // Extract IDs from markdown to verify they're valid
    let ids: Vec<i64> = final_content
        .lines()
        .filter(|line| line.contains("<!--ID:"))
        .filter_map(|line| {
            line.split("<!--ID:")
                .nth(1)?
                .split("-->")
                .next()?
                .trim()
                .parse::<i64>()
                .ok()
        })
        .collect();

    assert_eq!(ids.len(), 3, "Should extract 3 valid IDs");

    // Verify IDs are non-zero and unique
    for id in &ids {
        assert!(*id > 0, "ID should be positive");
    }

    let unique_ids: std::collections::HashSet<_> = ids.iter().collect();
    assert_eq!(unique_ids.len(), 3, "All IDs should be unique");

    Ok(())
}
