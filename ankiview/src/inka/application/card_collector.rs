use crate::infrastructure::anki::AnkiRepository;
use crate::inka::infrastructure::file_writer;
use crate::inka::infrastructure::hasher::HashCache;
use crate::inka::infrastructure::markdown::card_parser;
use crate::inka::infrastructure::markdown::converter;
use crate::inka::infrastructure::markdown::section_parser;
use crate::inka::infrastructure::media_handler;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::debug;

/// Main use case for collecting markdown cards into Anki
pub struct CardCollector {
    _collection_path: PathBuf,
    media_dir: PathBuf,
    repository: AnkiRepository,
    force: bool,
    hash_cache: Option<HashCache>,
    update_ids: bool,
    ignore_errors: bool,
    errors: Vec<String>,
}

impl CardCollector {
    /// Create a new CardCollector with Anki collection path
    pub fn new(
        collection_path: impl AsRef<Path>,
        force: bool,
        full_sync: bool,
        update_ids: bool,
        ignore_errors: bool,
    ) -> Result<Self> {
        let collection_path = collection_path.as_ref().to_path_buf();

        // Determine media directory path
        let media_dir = collection_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid collection path"))?
            .join("collection.media");

        // Create media directory if it doesn't exist
        if !media_dir.exists() {
            std::fs::create_dir_all(&media_dir).context("Failed to create media directory")?;
        }

        // Determine hash cache path (in same directory as collection)
        let cache_path = collection_path
            .parent()
            .expect("Invalid collection path")
            .join("ankiview_hashes.json");

        // Load hash cache unless full_sync is enabled
        let hash_cache = if full_sync {
            None
        } else {
            Some(HashCache::load(&cache_path).context("Failed to load hash cache")?)
        };

        // Open repository
        let repository = AnkiRepository::new(&collection_path)?;

        Ok(Self {
            _collection_path: collection_path,
            media_dir,
            repository,
            force,
            hash_cache,
            update_ids,
            ignore_errors,
            errors: Vec::new(),
        })
    }

    /// Get accumulated errors from processing
    pub fn errors(&self) -> &[String] {
        &self.errors
    }

    /// Process a single markdown file and add/update cards in Anki
    /// Returns the number of cards processed
    pub fn process_file(&mut self, markdown_path: impl AsRef<Path>) -> Result<usize> {
        let markdown_path = markdown_path.as_ref();

        // Handle error according to ignore_errors flag
        match self.process_file_impl(markdown_path) {
            Ok(count) => Ok(count),
            Err(e) => {
                if self.ignore_errors {
                    // Collect error and continue
                    let error_msg = format!("{}: {:#}", markdown_path.display(), e);
                    self.errors.push(error_msg);
                    Ok(0)
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Internal implementation of process_file
    fn process_file_impl(&mut self, markdown_path: &Path) -> Result<usize> {
        // Check if file has changed (skip if unchanged and cache exists)
        if let Some(cache) = &self.hash_cache {
            let has_changed = cache
                .file_has_changed(markdown_path)
                .context("Failed to check file hash")?;

            if !has_changed {
                // File unchanged, skip processing
                debug!(?markdown_path, "Skipping unchanged file");
                return Ok(0);
            }
        }

        // Read markdown file
        let mut content = file_writer::read_markdown_file(markdown_path)?;

        // Extract and handle media files
        let image_paths = media_handler::extract_image_paths(&content);
        let mut path_mapping = HashMap::new();

        for image_path in image_paths {
            // Resolve relative paths relative to markdown file location
            let markdown_dir = markdown_path
                .parent()
                .ok_or_else(|| anyhow::anyhow!("Cannot determine markdown file directory"))?;
            let absolute_image_path = markdown_dir.join(&image_path);

            // Copy image to media directory
            match media_handler::copy_media_to_anki(
                &absolute_image_path,
                &self.media_dir,
                self.force,
            ) {
                Ok(filename) => {
                    debug!("Copied media file: {} -> {}", image_path, filename);
                    path_mapping.insert(image_path.clone(), filename);
                }
                Err(e) => {
                    return Err(e)
                        .with_context(|| format!("Failed to copy media file '{}'", image_path));
                }
            }
        }

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
            let deck_name =
                section_parser::extract_deck_name(section).unwrap_or_else(|| "Default".to_string());
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
                    let mut front_html = converter::markdown_to_html(&front_md);
                    let mut back_html = converter::markdown_to_html(&back_md);

                    // Update media paths in HTML
                    front_html =
                        media_handler::update_media_paths_in_html(&front_html, &path_mapping);
                    back_html =
                        media_handler::update_media_paths_in_html(&back_html, &path_mapping);

                    // Create or update note
                    if let Some(id) = existing_id {
                        // Update existing note
                        self.repository
                            .update_note(id, &[front_html.clone(), back_html.clone()])?;
                    } else if self.update_ids {
                        // --update-ids mode: search for existing note by HTML content
                        let matching_ids = self
                            .repository
                            .search_by_html(&[front_html.clone(), back_html.clone()])?;

                        if let Some(&id) = matching_ids.first() {
                            // Found existing note, inject ID
                            debug!(note_id = id, "Found existing note for card, injecting ID");
                            content = file_writer::inject_anki_id(&content, &note_str, id);
                            // Update the existing note with current content
                            self.repository.update_note(id, &[front_html, back_html])?;
                        } else {
                            // No match found, create new note
                            let id = self.repository.create_basic_note(
                                &front_html,
                                &back_html,
                                &deck_name,
                                &tags,
                            )?;
                            content = file_writer::inject_anki_id(&content, &note_str, id);
                        }
                    } else {
                        // Normal mode: create new note
                        let id = self.repository.create_basic_note(
                            &front_html,
                            &back_html,
                            &deck_name,
                            &tags,
                        )?;

                        // Inject ID back into markdown
                        content = file_writer::inject_anki_id(&content, &note_str, id);
                    };

                    card_count += 1;
                } else if card_parser::is_cloze_card(&note_str) {
                    // Parse cloze card
                    let text_md = card_parser::parse_cloze_card_field(&note_str)?;

                    // Transform cloze syntax
                    let text_transformed = crate::inka::infrastructure::markdown::cloze_converter::convert_cloze_syntax(&text_md);

                    // Convert to HTML
                    let mut text_html = converter::markdown_to_html(&text_transformed);

                    // Update media paths in HTML
                    text_html =
                        media_handler::update_media_paths_in_html(&text_html, &path_mapping);

                    // Create or update note
                    if let Some(id) = existing_id {
                        // Update existing note
                        self.repository.update_note(id, &[text_html.clone()])?;
                    } else if self.update_ids {
                        // --update-ids mode: search for existing note by HTML content
                        let matching_ids = self.repository.search_by_html(&[text_html.clone()])?;

                        if let Some(&id) = matching_ids.first() {
                            // Found existing note, inject ID
                            debug!(
                                note_id = id,
                                "Found existing note for cloze card, injecting ID"
                            );
                            content = file_writer::inject_anki_id(&content, &note_str, id);
                            // Update the existing note with current content
                            self.repository.update_note(id, &[text_html])?;
                        } else {
                            // No match found, create new note
                            let id = self
                                .repository
                                .create_cloze_note(&text_html, &deck_name, &tags)?;
                            content = file_writer::inject_anki_id(&content, &note_str, id);
                        }
                    } else {
                        // Normal mode: create new note
                        let id = self
                            .repository
                            .create_cloze_note(&text_html, &deck_name, &tags)?;

                        // Inject ID back into markdown
                        content = file_writer::inject_anki_id(&content, &note_str, id);
                    };

                    card_count += 1;
                }
            }
        }

        // Write updated content back to file if IDs were injected
        file_writer::write_markdown_file(markdown_path, &content)?;

        // After successful processing, update hash cache
        if let Some(cache) = &mut self.hash_cache {
            cache
                .update_hash(markdown_path)
                .context("Failed to update file hash")?;
        }

        Ok(card_count)
    }

    /// Process a directory recursively
    /// Returns the number of cards processed
    pub fn process_directory(&mut self, dir_path: impl AsRef<Path>) -> Result<usize> {
        let dir_path = dir_path.as_ref();

        if !dir_path.is_dir() {
            return Err(anyhow::anyhow!("Path is not a directory: {:?}", dir_path));
        }

        let mut total_count = 0;

        // Walk directory recursively
        for entry in walkdir::WalkDir::new(dir_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Only process markdown files
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                total_count += self.process_file(path)?;
            }
        }

        Ok(total_count)
    }
}

impl Drop for CardCollector {
    fn drop(&mut self) {
        // Save hash cache if it exists
        if let Some(cache) = &self.hash_cache {
            if let Err(e) = cache.save() {
                // Use eprintln since we can't return Result from Drop
                eprintln!("Warning: Failed to save hash cache: {}", e);
            }
        }
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
            .join("tests/fixtures/test_collection/User 1/collection.anki2");
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

        let mut collector =
            CardCollector::new(&collection_path, false, false, false, false).unwrap();
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

        let mut collector =
            CardCollector::new(&collection_path, false, false, false, false).unwrap();
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

        let mut collector =
            CardCollector::new(&collection_path, false, false, false, false).unwrap();
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

        let mut collector =
            CardCollector::new(&collection_path, false, false, false, false).unwrap();

        // First run creates note
        let count1 = collector.process_file(&markdown_path).unwrap();
        assert_eq!(count1, 1);

        // Markdown should now have ID
        let updated_content = fs::read_to_string(&markdown_path).unwrap();
        assert!(updated_content.contains("<!--ID:"));

        // Modify the answer
        let modified = updated_content.replace(
            "A systems programming language",
            "A safe systems programming language",
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

        let mut collector =
            CardCollector::new(&collection_path, false, false, false, false).unwrap();
        let count = collector.process_file(&markdown_path).unwrap();

        assert_eq!(count, 0);
    }

    #[test]
    fn given_directory_with_markdown_files_when_processing_recursively_then_processes_all() {
        let (temp_dir, collection_path, _media_dir) = create_test_collection();

        // Create directory structure with markdown files
        let notes_dir = temp_dir.path().join("notes");
        fs::create_dir(&notes_dir).unwrap();

        let subdir = notes_dir.join("subdirectory");
        fs::create_dir(&subdir).unwrap();

        // File 1 in root notes dir
        let file1 = notes_dir.join("file1.md");
        fs::write(
            &file1,
            r#"---
Deck: Test

1. Question 1?
> Answer 1
---"#,
        )
        .unwrap();

        // File 2 in subdirectory
        let file2 = subdir.join("file2.md");
        fs::write(
            &file2,
            r#"---
Deck: Test

1. Question 2?
> Answer 2
---"#,
        )
        .unwrap();

        // Non-markdown file (should be ignored)
        let txt_file = notes_dir.join("readme.txt");
        fs::write(&txt_file, "This is not markdown").unwrap();

        let mut collector =
            CardCollector::new(&collection_path, false, false, false, false).unwrap();
        let count = collector.process_directory(&notes_dir).unwrap();

        // Should process both markdown files
        assert_eq!(count, 2);
    }

    #[test]
    fn given_ignore_errors_when_processing_file_with_missing_media_then_collects_error() {
        let (temp_dir, collection_path, _media_dir) = create_test_collection();

        // Create markdown with reference to non-existent image
        let markdown_path = temp_dir.path().join("missing_media.md");
        let markdown_content = r#"---
Deck: TestDeck

1. What is this image?
> ![missing image](images/nonexistent.png)
---"#;
        fs::write(&markdown_path, markdown_content).unwrap();

        // Process with ignore_errors = true
        let mut collector =
            CardCollector::new(&collection_path, false, false, false, true).unwrap();
        let count = collector.process_file(&markdown_path).unwrap();

        // Should return 0 cards since processing failed
        assert_eq!(count, 0);

        // Should have collected the error
        let errors = collector.errors();
        assert_eq!(errors.len(), 1, "Should have 1 error");
        assert!(
            errors[0].contains("missing_media.md"),
            "Error message should mention the file"
        );
    }

    #[test]
    fn given_no_ignore_errors_when_processing_file_with_missing_media_then_returns_error() {
        let (temp_dir, collection_path, _media_dir) = create_test_collection();

        // Create markdown with reference to non-existent image
        let markdown_path = temp_dir.path().join("missing_media.md");
        let markdown_content = r#"---
Deck: TestDeck

1. What is this image?
> ![missing image](images/nonexistent.png)
---"#;
        fs::write(&markdown_path, markdown_content).unwrap();

        // Process with ignore_errors = false
        let mut collector =
            CardCollector::new(&collection_path, false, false, false, false).unwrap();
        let result = collector.process_file(&markdown_path);

        // Should return an error
        assert!(result.is_err(), "Should return an error");

        // Should not have collected any errors (since we returned immediately)
        let errors = collector.errors();
        assert_eq!(errors.len(), 0, "Should have 0 collected errors");
    }

    #[test]
    fn given_markdown_with_image_when_processing_then_copies_media_file() {
        let (temp_dir, collection_path, media_dir) = create_test_collection();

        // Create a test image file
        let images_dir = temp_dir.path().join("images");
        fs::create_dir(&images_dir).unwrap();
        let source_image = images_dir.join("test_photo.png");
        fs::write(&source_image, b"fake png data").unwrap();

        // Create markdown with image reference
        let markdown_path = temp_dir.path().join("with_image.md");
        let markdown_content = r#"---
Deck: TestDeck

1. What is this image?
> ![test image](images/test_photo.png)
> This is a test
---"#;
        fs::write(&markdown_path, markdown_content).unwrap();

        // Process the file
        let mut collector =
            CardCollector::new(&collection_path, false, false, false, false).unwrap();
        let count = collector.process_file(&markdown_path).unwrap();

        assert_eq!(count, 1);

        // Verify image was copied to media directory
        let copied_image = media_dir.join("test_photo.png");
        assert!(
            copied_image.exists(),
            "Image should be copied to media directory"
        );

        // Verify image content is correct
        let copied_content = fs::read(&copied_image).unwrap();
        assert_eq!(copied_content, b"fake png data");
    }
}
