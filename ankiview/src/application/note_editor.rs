// src/application/note_editor.rs
use crate::application::NoteRepository;
use crate::infrastructure::note_template::NoteTemplate;
use anyhow::{Context, Result};
use std::fs;
use std::process::Command;
use tracing::{debug, info};

pub struct NoteEditor<R: NoteRepository> {
    repository: R,
}

impl<R: NoteRepository> NoteEditor<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn edit(&mut self, note_id: i64) -> Result<bool> {
        // Fetch the note
        let note = self
            .repository
            .get_note(note_id)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        // Build template from note
        let template = NoteTemplate::from_note(&note);
        let template_text = template.to_string();

        // Write to temp file
        let temp_file = tempfile::Builder::new()
            .suffix(".md")
            .tempfile()
            .context("Failed to create temporary file")?;

        fs::write(temp_file.path(), &template_text)
            .context("Failed to write template to temp file")?;

        // Record modification time before editor
        let modified_before = fs::metadata(temp_file.path())?.modified()?;

        // Open editor
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
        debug!(editor = %editor, path = ?temp_file.path(), "Opening editor");

        let status = Command::new(&editor)
            .arg(temp_file.path())
            .status()
            .with_context(|| {
                format!(
                    "Failed to open editor '{}'. Set the EDITOR environment variable.",
                    editor
                )
            })?;

        if !status.success() {
            return Err(anyhow::anyhow!(
                "Editor exited with non-zero status (code {})",
                status.code().unwrap_or(-1)
            ));
        }

        // Check if file was modified
        let modified_after = fs::metadata(temp_file.path())?.modified()?;
        if modified_after <= modified_before {
            info!("No changes detected.");
            return Ok(false);
        }

        // Read edited content
        let edited_text =
            fs::read_to_string(temp_file.path()).context("Failed to read edited template")?;

        // Parse and validate
        let edited_template = NoteTemplate::from_string(&edited_text, &note)?;
        edited_template.validate(&note)?;

        // Apply changes
        let (fields, tags) = edited_template.to_update();
        self.repository
            .update_note_fields_and_tags(note_id, &fields, &tags)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        info!(note_id, "Note updated successfully.");
        Ok(true)
    }
}
