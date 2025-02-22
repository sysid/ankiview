// src/lib.rs
pub mod application;
pub mod cli;
pub mod domain;
pub mod infrastructure;
pub mod ports;
pub mod util;

use std::path::PathBuf;
use anyhow::{Context, Result};
use infrastructure::AnkiRepository;
use ports::HtmlPresenter;
use tracing::{debug, info};
use crate::cli::args::Args;

pub fn run(args: Args) -> Result<()> {
    debug!(?args, "Starting ankiview with arguments");

    // Initialize infrastructure
    let collection_path = match args.collection {
        Some(path) => {
            debug!(?path, "Using provided collection path");
            path
        }
        None => {
            debug!(?args.profile, "Finding collection path for profile");
            find_collection_path(args.profile.as_deref())?
        }
    };

    let repository = AnkiRepository::new(&collection_path)?;
    let media_dir = repository.media_dir().to_path_buf();

    // Initialize application
    let mut viewer = application::NoteViewer::new(repository);

    // Initialize presentation
    let presenter = HtmlPresenter::with_media_dir(media_dir);
    let mut renderer = infrastructure::renderer::ContentRenderer::new();

    // Execute use case
    info!(note_id = args.note_id, "Viewing note");
    let note = viewer.view_note(args.note_id)?;
    debug!(?note, "Retrieved note");

    let html = presenter.render(&note);
    debug!(?html, "Generated HTML");

    // Create temporary file and open in browser
    let temp_path = renderer.create_temp_file(&html)?;
    renderer.open_in_browser(&temp_path)?;

    Ok(())
}

pub fn find_collection_path(profile: Option<&str>) -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not find home directory")?;

    // Get the Anki base directory
    #[cfg(target_os = "macos")]
    let anki_path = home.join("Library/Application Support/Anki2");
    #[cfg(target_os = "linux")]
    let anki_path = home.join(".local/share/Anki2");
    #[cfg(target_os = "windows")]
    let anki_path = home.join("AppData/Roaming/Anki2");

    // If profile is specified, use it directly
    if let Some(profile_name) = profile {
        return Ok(anki_path.join(profile_name).join("collection.anki2"));
    }

    // Otherwise, find the first valid profile
    for entry in std::fs::read_dir(&anki_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && path.join("collection.anki2").exists() {
            return Ok(path.join("collection.anki2"));
        }
    }

    Err(anyhow::anyhow!("No valid Anki profile found"))
}

#[cfg(test)]
/// must be public to be used from integration tests
mod tests {
    use crate::util::testing;
    #[ctor::ctor]
    fn init() {
        testing::init_test_setup().expect("Failed to initialize test setup");
    }
}