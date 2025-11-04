// src/infrastructure/anki.rs
use crate::application::NoteRepository;
use crate::domain::{DomainError, Note};
use anki::collection::{Collection, CollectionBuilder};
use anki::notes::NoteId;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, instrument};

pub struct AnkiRepository {
    collection: Collection,
    media_dir: PathBuf,
}

impl AnkiRepository {
    pub fn new<P: AsRef<Path>>(collection_path: P) -> Result<Self> {
        let path = PathBuf::from(collection_path.as_ref());
        debug!(?path, "Creating new AnkiRepository");

        // FIRST: Check if Anki is running before attempting anything else
        // This prevents database corruption from concurrent access
        crate::util::process::check_anki_not_running()
            .context("Cannot access collection while Anki is running")?;

        // Check if file exists
        if !path.exists() {
            return Err(DomainError::CollectionError(format!(
                "Collection file not found: {}",
                path.display()
            ))
            .into());
        }

        // Check if we have read permissions
        match fs::metadata(&path) {
            Ok(metadata) => {
                if metadata.permissions().readonly() {
                    return Err(DomainError::CollectionError(format!(
                        "No write permission for collection: {}",
                        path.display()
                    ))
                    .into());
                }
            }
            Err(e) => {
                return Err(DomainError::CollectionError(format!(
                    "Failed to read collection metadata: {}",
                    e
                ))
                .into());
            }
        }

        // Try to open the collection with improved error context
        let collection = CollectionBuilder::new(path.clone())
            .build()
            .with_context(|| {
                "Failed to open Anki collection.\n\n\
                 Possible causes:\n\
                 - Anki is running (please close it completely)\n\
                 - Database is locked by another process\n\
                 - Collection file is corrupted\n\n\
                 If you just closed Anki, wait 5-10 seconds and try again."
            })?;

        // Get media directory path
        let media_dir = path.parent().unwrap().join("collection.media");

        info!(?path, "Successfully opened Anki collection");
        Ok(Self {
            collection,
            media_dir,
        })
    }

    pub fn media_dir(&self) -> &Path {
        &self.media_dir
    }

    /// Find or create a Basic note type with front/back fields
    /// Returns the notetype ID
    pub fn find_or_create_basic_notetype(&mut self) -> Result<i64> {
        use anki::notetype::NotetypeKind;

        // Look for existing Basic notetype
        let all_notetypes = self
            .collection
            .get_all_notetypes()
            .context("Failed to get all notetypes")?;

        // Find a Basic-type notetype (non-cloze)
        for notetype in all_notetypes {
            if notetype.config.kind() != NotetypeKind::Cloze && notetype.fields.len() >= 2 {
                // Found a suitable basic notetype
                debug!(notetype_id = notetype.id.0, name = %notetype.name, "Found existing Basic notetype");
                return Ok(notetype.id.0);
            }
        }

        // No suitable notetype found - this shouldn't happen in normal Anki collections
        // For now, return an error. In the future, we could create one programmatically.
        Err(anyhow::anyhow!(
            "No Basic notetype found. Please create a Basic notetype in Anki first."
        ))
    }

    /// Find or create a Cloze note type
    /// Returns the notetype ID
    pub fn find_or_create_cloze_notetype(&mut self) -> Result<i64> {
        use anki::notetype::NotetypeKind;

        // Look for existing Cloze notetype
        let all_notetypes = self
            .collection
            .get_all_notetypes()
            .context("Failed to get all notetypes")?;

        // Find a Cloze-type notetype
        for notetype in all_notetypes {
            if notetype.config.kind() == NotetypeKind::Cloze {
                // Found a cloze notetype
                debug!(notetype_id = notetype.id.0, name = %notetype.name, "Found existing Cloze notetype");
                return Ok(notetype.id.0);
            }
        }

        // No cloze notetype found - this shouldn't happen in normal Anki collections
        Err(anyhow::anyhow!(
            "No Cloze notetype found. Please create a Cloze notetype in Anki first."
        ))
    }

    /// Create a new Basic note in the collection
    /// Returns the created note ID
    pub fn create_basic_note(
        &mut self,
        front: &str,
        back: &str,
        deck_name: &str,
        tags: &[String],
    ) -> Result<i64> {
        use anki::notes::Note;
        use anki::notetype::NotetypeId;

        // Find or create the Basic notetype
        let notetype_id = self.find_or_create_basic_notetype()?;

        // Get the notetype to create the note
        let notetype = self
            .collection
            .get_notetype(NotetypeId(notetype_id))
            .context("Failed to get notetype")?
            .context("Notetype not found")?;

        // Find or create the deck
        let deck_id = self
            .collection
            .get_or_create_normal_deck(deck_name)
            .context("Failed to get or create deck")?
            .id;

        // Create a new note
        let mut note = Note::new(&notetype);
        note.set_field(0, front)
            .context("Failed to set front field")?;
        note.set_field(1, back)
            .context("Failed to set back field")?;

        // Add tags
        for tag in tags {
            note.tags.push(tag.clone());
        }

        // Add the note to the collection
        self.collection
            .add_note(&mut note, deck_id)
            .context("Failed to add note to collection")?;

        debug!(note_id = note.id.0, "Created Basic note");
        Ok(note.id.0)
    }

    /// Create a new Cloze note in the collection
    /// Returns the created note ID
    pub fn create_cloze_note(
        &mut self,
        text: &str,
        deck_name: &str,
        tags: &[String],
    ) -> Result<i64> {
        use anki::notes::Note;
        use anki::notetype::NotetypeId;

        // Find or create the Cloze notetype
        let notetype_id = self.find_or_create_cloze_notetype()?;

        // Get the notetype to create the note
        let notetype = self
            .collection
            .get_notetype(NotetypeId(notetype_id))
            .context("Failed to get notetype")?
            .context("Notetype not found")?;

        // Find or create the deck
        let deck_id = self
            .collection
            .get_or_create_normal_deck(deck_name)
            .context("Failed to get or create deck")?
            .id;

        // Create a new note
        let mut note = Note::new(&notetype);
        note.set_field(0, text)
            .context("Failed to set text field")?;

        // Add tags
        for tag in tags {
            note.tags.push(tag.clone());
        }

        // Add the note to the collection
        self.collection
            .add_note(&mut note, deck_id)
            .context("Failed to add note to collection")?;

        debug!(note_id = note.id.0, "Created Cloze note");
        Ok(note.id.0)
    }

    /// Update an existing note's fields
    /// For Basic notes: updates front (field 0) and back (field 1)
    /// For Cloze notes: updates text (field 0)
    pub fn update_note(&mut self, note_id: i64, fields: &[String]) -> Result<()> {
        use anki::notes::NoteId;

        // Get the existing note
        let mut note = self
            .collection
            .storage
            .get_note(NoteId(note_id))
            .context("Failed to get note from storage")?
            .ok_or_else(|| anyhow::anyhow!("Note not found: {}", note_id))?;

        // Update each field
        for (index, field_value) in fields.iter().enumerate() {
            note.set_field(index, field_value)
                .with_context(|| format!("Failed to set field {} on note {}", index, note_id))?;
        }

        // Save the updated note
        self.collection
            .update_note(&mut note)
            .context("Failed to update note in collection")?;

        debug!(note_id, "Updated note fields");
        Ok(())
    }

    /// Check if a note exists by ID
    pub fn note_exists(&self, note_id: i64) -> Result<bool> {
        use anki::notes::NoteId;

        let exists = self
            .collection
            .storage
            .get_note(NoteId(note_id))
            .context("Failed to check note existence")?
            .is_some();

        Ok(exists)
    }

    /// Search for notes by HTML content (for --update-ids)
    /// Returns a vector of note IDs that match the given HTML fields
    pub fn search_by_html(&mut self, fields: &[String]) -> Result<Vec<i64>> {
        use anki::search::SearchNode;

        // Get all notes in the collection
        let search_node = SearchNode::WholeCollection;
        let note_ids = self
            .collection
            .search_notes_unordered(search_node)
            .context("Failed to search notes")?;

        let mut matching_ids = Vec::new();

        // Check each note to see if its fields match
        for note_id in note_ids {
            if let Ok(Some(note)) = self.collection.storage.get_note(note_id) {
                let note_fields: Vec<String> =
                    note.fields().iter().map(|f| f.to_string()).collect();

                // For basic cards, match front and back (first 2 fields)
                // For cloze cards, match the text field (first field)
                let matches = if fields.len() == 2 && note_fields.len() >= 2 {
                    // Basic card: match both fields
                    note_fields[0] == fields[0] && note_fields[1] == fields[1]
                } else if fields.len() == 1 && !note_fields.is_empty() {
                    // Cloze card: match first field
                    note_fields[0] == fields[0]
                } else {
                    false
                };

                if matches {
                    debug!(note_id = note_id.0, "Found matching note");
                    matching_ids.push(note_id.0);
                }
            }
        }

        Ok(matching_ids)
    }
}

impl NoteRepository for AnkiRepository {
    #[instrument(level = "debug", skip(self))]
    fn get_note(&mut self, id: i64) -> Result<Note, DomainError> {
        let note = self
            .collection
            .storage
            .get_note(NoteId(id))
            .map_err(|_| DomainError::NoteNotFound(id))?
            .ok_or(DomainError::NoteNotFound(id))?;

        let model = self
            .collection
            .get_notetype(note.notetype_id)
            .map_err(|e| DomainError::CollectionError(e.to_string()))?
            .ok_or_else(|| DomainError::CollectionError("Notetype not found".to_string()))?;

        let fields: Vec<_> = note.fields().iter().map(|f| f.to_string()).collect();

        Ok(Note {
            id: note.id.0,
            front: fields.first().cloned().unwrap_or_default(),
            back: fields.get(1).cloned().unwrap_or_default(),
            tags: note.tags.to_vec(),
            model_name: model.name.clone(),
        })
    }

    #[instrument(level = "debug", skip(self))]
    fn delete_note(&mut self, id: i64) -> Result<usize, DomainError> {
        debug!(note_id = id, "Attempting to delete note");

        // Check if note exists first to provide better error messages
        let note_exists = self
            .collection
            .storage
            .get_note(NoteId(id))
            .map_err(|e| {
                DomainError::CollectionError(format!("Failed to check note existence: {}", e))
            })?
            .is_some();

        if !note_exists {
            debug!(note_id = id, "Note not found for deletion");
            return Err(DomainError::NoteNotFound(id));
        }

        // Delete the note using the public API
        // This handles cascading card deletion automatically
        let result = self
            .collection
            .remove_notes(&[NoteId(id)])
            .map_err(|e| DomainError::CollectionError(format!("Failed to delete note: {}", e)))?;

        // Extract the count of deleted cards from OpOutput
        let deleted_card_count = result.output;

        info!(
            note_id = id,
            cards_deleted = deleted_card_count,
            "Successfully deleted note"
        );

        Ok(deleted_card_count)
    }

    #[instrument(level = "debug", skip(self))]
    fn list_notes(&mut self, search_query: Option<&str>) -> Result<Vec<Note>, DomainError> {
        // Get note IDs based on search query
        let note_ids: Vec<NoteId> = match search_query {
            None => {
                // No search - get all notes (fastest method)
                self.collection
                    .storage
                    .get_all_note_ids()
                    .map_err(|e| DomainError::CollectionError(e.to_string()))?
                    .into_iter()
                    .collect()
            }
            Some(query) => {
                // Build search query for front field
                let search_str = if query.is_empty() {
                    // Empty query string = all notes
                    "".to_string()
                } else {
                    // Search in front field for the query string
                    format!("front:*{}*", query)
                };

                // Use unordered search (faster, no sort needed)
                self.collection
                    .search_notes_unordered(&search_str)
                    .map_err(|e| DomainError::CollectionError(e.to_string()))?
            }
        };

        // Fetch full note data for each ID
        let mut notes = Vec::new();
        for note_id in note_ids {
            // Use existing get_note logic
            match self.get_note(note_id.0) {
                Ok(note) => notes.push(note),
                Err(DomainError::NoteNotFound(_)) => {
                    // Skip notes that don't exist (race condition or corrupted DB)
                    debug!(note_id = note_id.0, "Skipping note that doesn't exist");
                    continue;
                }
                Err(e) => return Err(e), // Propagate other errors
            }
        }

        Ok(notes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Helper to create a temporary test collection
    fn create_test_collection() -> Result<(TempDir, AnkiRepository)> {
        let temp_dir = TempDir::new()?;
        let collection_path = temp_dir.path().join("collection.anki2");

        // Create a new Anki collection
        let collection = CollectionBuilder::new(&collection_path).build()?;
        drop(collection); // Close it

        // Open it with our repository
        let repo = AnkiRepository::new(&collection_path)?;

        Ok((temp_dir, repo))
    }

    #[test]
    fn given_new_collection_when_finding_basic_notetype_then_creates_and_returns_id() {
        let (_temp_dir, mut repo) = create_test_collection().unwrap();

        let notetype_id = repo.find_or_create_basic_notetype().unwrap();

        assert!(notetype_id > 0);
    }

    #[test]
    fn given_existing_basic_notetype_when_finding_then_returns_same_id() {
        let (_temp_dir, mut repo) = create_test_collection().unwrap();

        let first_id = repo.find_or_create_basic_notetype().unwrap();
        let second_id = repo.find_or_create_basic_notetype().unwrap();

        assert_eq!(first_id, second_id);
    }

    #[test]
    fn given_new_collection_when_finding_cloze_notetype_then_creates_and_returns_id() {
        let (_temp_dir, mut repo) = create_test_collection().unwrap();

        let notetype_id = repo.find_or_create_cloze_notetype().unwrap();

        assert!(notetype_id > 0);
    }

    #[test]
    fn given_existing_cloze_notetype_when_finding_then_returns_same_id() {
        let (_temp_dir, mut repo) = create_test_collection().unwrap();

        let first_id = repo.find_or_create_cloze_notetype().unwrap();
        let second_id = repo.find_or_create_cloze_notetype().unwrap();

        assert_eq!(first_id, second_id);
    }

    #[test]
    fn given_basic_card_fields_when_creating_note_then_returns_note_id() {
        let (_temp_dir, mut repo) = create_test_collection().unwrap();

        let note_id = repo
            .create_basic_note(
                "What is Rust?",
                "A systems programming language",
                "Default",
                &["rust".to_string(), "programming".to_string()],
            )
            .unwrap();

        assert!(note_id > 0);
    }

    #[test]
    fn given_basic_note_when_created_then_can_retrieve() {
        let (_temp_dir, mut repo) = create_test_collection().unwrap();

        let note_id = repo
            .create_basic_note("Front", "Back", "Default", &[])
            .unwrap();

        // Should be able to retrieve the note
        let note = repo.get_note(note_id).unwrap();
        assert_eq!(note.id, note_id);
        assert!(note.front.contains("Front"));
        assert!(note.back.contains("Back"));
    }

    #[test]
    fn given_cloze_text_when_creating_note_then_returns_note_id() {
        let (_temp_dir, mut repo) = create_test_collection().unwrap();

        let note_id = repo
            .create_cloze_note(
                "The capital of {{c1::France}} is {{c2::Paris}}",
                "Default",
                &["geography".to_string()],
            )
            .unwrap();

        assert!(note_id > 0);
    }

    #[test]
    fn given_cloze_note_when_created_then_can_retrieve() {
        let (_temp_dir, mut repo) = create_test_collection().unwrap();

        let cloze_text = "Answer: {{c1::42}}";
        let note_id = repo.create_cloze_note(cloze_text, "Default", &[]).unwrap();

        // Should be able to retrieve the note
        let note = repo.get_note(note_id).unwrap();
        assert_eq!(note.id, note_id);
        assert!(note.front.contains("42"));
    }

    #[test]
    fn given_existing_note_when_updating_then_fields_change() {
        let (_temp_dir, mut repo) = create_test_collection().unwrap();

        // Create a note
        let note_id = repo
            .create_basic_note("Original Front", "Original Back", "Default", &[])
            .unwrap();

        // Update it
        let new_fields = vec!["Updated Front".to_string(), "Updated Back".to_string()];
        repo.update_note(note_id, &new_fields).unwrap();

        // Retrieve and verify
        let note = repo.get_note(note_id).unwrap();
        assert!(note.front.contains("Updated Front"));
        assert!(note.back.contains("Updated Back"));
    }

    #[test]
    fn given_nonexistent_note_when_updating_then_returns_error() {
        let (_temp_dir, mut repo) = create_test_collection().unwrap();

        let result = repo.update_note(9999999, &["Test".to_string()]);

        assert!(result.is_err());
    }

    #[test]
    fn given_existing_note_when_checking_exists_then_returns_true() {
        let (_temp_dir, mut repo) = create_test_collection().unwrap();

        let note_id = repo
            .create_basic_note("Front", "Back", "Default", &[])
            .unwrap();

        assert!(repo.note_exists(note_id).unwrap());
    }

    #[test]
    fn given_nonexistent_note_when_checking_exists_then_returns_false() {
        let (_temp_dir, repo) = create_test_collection().unwrap();

        assert!(!repo.note_exists(9999999).unwrap());
    }
}
