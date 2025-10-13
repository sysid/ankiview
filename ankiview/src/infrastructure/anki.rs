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
}
