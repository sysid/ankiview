use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use crate::infrastructure::anki::AnkiRepository;
use crate::inka::infrastructure::markdown::section_parser;
use crate::inka::infrastructure::markdown::card_parser;
use crate::inka::infrastructure::markdown::converter;
use crate::inka::infrastructure::file_writer;

/// Main use case for collecting markdown cards into Anki
pub struct CardCollector {
    collection_path: PathBuf,
    media_dir: PathBuf,
    repository: AnkiRepository,
}

impl CardCollector {
    /// Create a new CardCollector with Anki collection path
    pub fn new(collection_path: impl AsRef<Path>) -> Result<Self> {
        let collection_path = collection_path.as_ref().to_path_buf();

        // Determine media directory path
        let media_dir = collection_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid collection path"))?
            .join("collection.media");

        // Create media directory if it doesn't exist
        if !media_dir.exists() {
            std::fs::create_dir_all(&media_dir)
                .context("Failed to create media directory")?;
        }

        // Open repository
        let repository = AnkiRepository::new(&collection_path)?;

        Ok(Self {
            collection_path,
            media_dir,
            repository,
        })
    }

    /// Process a single markdown file and add/update cards in Anki
    /// Returns the number of cards processed
    pub fn process_file(&mut self, markdown_path: impl AsRef<Path>) -> Result<usize> {
        let markdown_path = markdown_path.as_ref();

        // Read markdown file
        let mut content = file_writer::read_markdown_file(markdown_path)?;

        // Parse sections
        let parser = section_parser::SectionParser::new();
        let sections = parser.parse(&content);

        if sections.is_empty() {
            return Ok(0);
        }

        // Convert sections to owned Strings to avoid borrowing issues when mutating content
        let sections: Vec<String> = sections.iter().map(|s| s.to_string()).collect();

        let mut card_count = 0;

        for section in &sections {
            // Extract metadata
            let deck_name = section_parser::extract_deck_name(section)
                .unwrap_or_else(|| "Default".to_string());
            let tags = section_parser::extract_tags(section);

            // Extract note strings
            let note_strings = section_parser::extract_note_strings(section);

            for note_str in note_strings {
                // Extract existing ID if present
                let existing_id = card_parser::extract_anki_id(&note_str);

                // Determine card type and process
                if card_parser::is_basic_card(&note_str) {
                    // Parse basic card fields
                    let (front_md, back_md) = card_parser::parse_basic_card_fields(&note_str)?;

                    // Convert to HTML
                    let front_html = converter::markdown_to_html(&front_md);
                    let back_html = converter::markdown_to_html(&back_md);

                    // Create or update note
                    let note_id = if let Some(id) = existing_id {
                        // Update existing note
                        self.repository.update_note(id, &[front_html, back_html])?;
                        id
                    } else {
                        // Create new note
                        let id = self.repository.create_basic_note(
                            &front_html,
                            &back_html,
                            &deck_name,
                            &tags,
                        )?;

                        // Inject ID back into markdown
                        content = file_writer::inject_anki_id(&content, &note_str, id);
                        id
                    };

                    card_count += 1;

                } else if card_parser::is_cloze_card(&note_str) {
                    // Parse cloze card
                    let text_md = card_parser::parse_cloze_card_field(&note_str)?;

                    // Transform cloze syntax
                    let text_transformed = crate::inka::infrastructure::markdown::cloze_converter::convert_cloze_syntax(&text_md);

                    // Convert to HTML
                    let text_html = converter::markdown_to_html(&text_transformed);

                    // Create or update note
                    let note_id = if let Some(id) = existing_id {
                        // Update existing note
                        self.repository.update_note(id, &[text_html])?;
                        id
                    } else {
                        // Create new note
                        let id = self.repository.create_cloze_note(
                            &text_html,
                            &deck_name,
                            &tags,
                        )?;

                        // Inject ID back into markdown
                        content = file_writer::inject_anki_id(&content, &note_str, id);
                        id
                    };

                    card_count += 1;
                }
            }
        }

        // Write updated content back to file if IDs were injected
        file_writer::write_markdown_file(markdown_path, &content)?;

        Ok(card_count)
    }

    /// Process a directory recursively
    /// Returns the number of cards processed
    pub fn process_directory(&mut self, dir_path: impl AsRef<Path>) -> Result<usize> {
        todo!("Implement CardCollector::process_directory")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // Test helper that creates a temporary test collection
    fn create_test_collection() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
        use std::path::PathBuf;
        let temp_dir = tempfile::tempdir().unwrap();

        let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/test_collection/collection.anki2");
        let collection_path = temp_dir.path().join("collection.anki2");

        std::fs::copy(&fixture_path, &collection_path).unwrap();

        let media_dir = temp_dir.path().join("collection.media");
        std::fs::create_dir_all(&media_dir).unwrap();

        (temp_dir, collection_path, media_dir)
    }

    #[test]
    fn given_markdown_with_basic_card_when_processing_then_creates_note() {
        let (temp_dir, collection_path, _media_dir) = create_test_collection();

        let markdown_path = temp_dir.path().join("test.md");
        let markdown_content = r#"---
Deck: TestDeck

1. What is Rust?
> A systems programming language
---"#;
        fs::write(&markdown_path, markdown_content).unwrap();

        let mut collector = CardCollector::new(&collection_path).unwrap();
        let count = collector.process_file(&markdown_path).unwrap();

        assert_eq!(count, 1);
    }

    #[test]
    fn given_markdown_with_cloze_card_when_processing_then_creates_note() {
        let (temp_dir, collection_path, _media_dir) = create_test_collection();

        let markdown_path = temp_dir.path().join("cloze.md");
        let markdown_content = r#"---
Deck: TestDeck

1. Rust is a {systems programming} language.
---"#;
        fs::write(&markdown_path, markdown_content).unwrap();

        let mut collector = CardCollector::new(&collection_path).unwrap();
        let count = collector.process_file(&markdown_path).unwrap();

        assert_eq!(count, 1);
    }

    #[test]
    fn given_markdown_with_multiple_cards_when_processing_then_creates_all() {
        let (temp_dir, collection_path, _media_dir) = create_test_collection();

        let markdown_path = temp_dir.path().join("multi.md");
        let markdown_content = r#"---
Deck: TestDeck

1. What is Rust?
> A systems programming language

2. What is Cargo?
> Rust's package manager

3. Rust was created by {Mozilla}.
---"#;
        fs::write(&markdown_path, markdown_content).unwrap();

        let mut collector = CardCollector::new(&collection_path).unwrap();
        let count = collector.process_file(&markdown_path).unwrap();

        assert_eq!(count, 3);
    }

    #[test]
    fn given_markdown_with_id_when_processing_second_time_then_updates_note() {
        let (temp_dir, collection_path, _media_dir) = create_test_collection();

        let markdown_path = temp_dir.path().join("update.md");
        let markdown_content = r#"---
Deck: TestDeck

1. What is Rust?
> A systems programming language
---"#;
        fs::write(&markdown_path, markdown_content).unwrap();

        let mut collector = CardCollector::new(&collection_path).unwrap();

        // First run creates note
        let count1 = collector.process_file(&markdown_path).unwrap();
        assert_eq!(count1, 1);

        // Markdown should now have ID
        let updated_content = fs::read_to_string(&markdown_path).unwrap();
        assert!(updated_content.contains("<!--ID:"));

        // Modify the answer
        let modified = updated_content.replace(
            "A systems programming language",
            "A safe systems programming language"
        );
        fs::write(&markdown_path, &modified).unwrap();

        // Second run updates note
        let count2 = collector.process_file(&markdown_path).unwrap();
        assert_eq!(count2, 1);
    }

    #[test]
    fn given_empty_markdown_when_processing_then_returns_zero() {
        let (temp_dir, collection_path, _media_dir) = create_test_collection();

        let markdown_path = temp_dir.path().join("empty.md");
        fs::write(&markdown_path, "Just text, no sections").unwrap();

        let mut collector = CardCollector::new(&collection_path).unwrap();
        let count = collector.process_file(&markdown_path).unwrap();

        assert_eq!(count, 0);
    }
}
