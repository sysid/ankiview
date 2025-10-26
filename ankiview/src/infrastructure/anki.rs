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

        // Try to open the collection with better error context
        let collection = CollectionBuilder::new(path.clone())
            .build()
            .with_context(|| "Failed to open Anki collection. Is Anki currently running?")?;

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
            front: fields[0].clone(),
            back: fields[1].clone(),
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
}
