// src/lib.rs
pub mod application;
pub mod cli;
pub mod domain;
pub mod infrastructure;
pub mod inka;
pub mod ports;
pub mod util;

use crate::application::NoteRepository;
use crate::cli::args::{Args, Command};
use anyhow::{Context, Result};
use infrastructure::AnkiRepository;
use ports::HtmlPresenter;
use std::path::PathBuf;
use tracing::{debug, info};

pub fn run(args: Args) -> Result<()> {
    debug!(?args, "Starting ankiview with arguments");

    // Resolve collection path from global flags
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

    // Route to appropriate handler based on command
    match args.command {
        Command::View { note_id, json } => handle_view_command(note_id, json, collection_path),
        Command::Delete { note_id } => handle_delete_command(note_id, collection_path),
        Command::List { search } => handle_list_command(search.as_deref(), collection_path),
        Command::Collect {
            path,
            recursive,
            force,
            ignore_errors,
            full_sync,
            update_ids,
            card_type,
        } => handle_collect_command(
            path,
            recursive,
            force,
            ignore_errors,
            full_sync,
            update_ids,
            card_type,
            collection_path,
        ),
        Command::ListCardTypes => handle_list_card_types_command(collection_path),
    }
}

fn handle_view_command(note_id: i64, json: bool, collection_path: PathBuf) -> Result<()> {
    let repository = AnkiRepository::new(&collection_path)?;
    let media_dir = repository.media_dir().to_path_buf();

    // Initialize application
    let mut viewer = application::NoteViewer::new(repository);

    // Execute use case
    info!(note_id = note_id, "Viewing note");
    let note = viewer.view_note(note_id)?;
    debug!(?note, "Retrieved note");

    // Branch on output format
    if json {
        // JSON output path
        let json_output =
            serde_json::to_string_pretty(&note).context("Failed to serialize note to JSON")?;
        println!("{}", json_output);
    } else {
        // Browser output path (existing behavior)
        let presenter = HtmlPresenter::with_media_dir(media_dir);
        let mut renderer = infrastructure::renderer::ContentRenderer::new();

        let html = presenter.render(&note);
        debug!(?html, "Generated HTML");

        // Create temporary file and open in browser
        let temp_path = renderer.create_temp_file(&html)?;
        renderer.open_in_browser(&temp_path)?;
    }

    Ok(())
}

fn handle_delete_command(note_id: i64, collection_path: PathBuf) -> Result<()> {
    let repository = AnkiRepository::new(&collection_path)?;

    // Initialize application
    let mut deleter = application::NoteDeleter::new(repository);

    // Execute use case
    info!(note_id = note_id, "Deleting note");
    let deleted_cards = deleter
        .delete_note(note_id)
        .with_context(|| format!("Failed to delete note {}", note_id))?;

    // Print success message to stdout (unlike view which is silent)
    println!(
        "Successfully deleted note {} ({} card{} removed)",
        note_id,
        deleted_cards,
        if deleted_cards == 1 { "" } else { "s" }
    );

    Ok(())
}

fn handle_list_command(search_query: Option<&str>, collection_path: PathBuf) -> Result<()> {
    let repository = AnkiRepository::new(&collection_path)?;

    // Initialize application
    let mut lister = application::NoteLister::new(repository);

    // Execute use case
    info!(?search_query, "Listing notes");
    let notes = lister.list_notes(search_query)?;
    debug!(note_count = notes.len(), "Retrieved notes");

    // Format and print output
    for note in notes {
        let first_line = util::text::extract_first_line(&note.front);
        println!("{}\t{}", note.id, first_line);
    }

    Ok(())
}

fn handle_list_card_types_command(collection_path: PathBuf) -> Result<()> {
    let mut repository = AnkiRepository::new(&collection_path)?;

    // List all available notetypes
    info!("Listing card types");
    let notetypes = repository.list_notetypes()?;
    debug!(count = notetypes.len(), "Retrieved notetypes");

    // Print header
    println!("Available card types:");
    println!("{:<15} {}", "ID", "Name");
    println!("{}", "-".repeat(60));

    // Format and print each notetype
    for (id, name) in notetypes {
        println!("{:<15} {}", id, name);
    }

    Ok(())
}

fn handle_collect_command(
    path: PathBuf,
    recursive: bool,
    force: bool,
    ignore_errors: bool,
    full_sync: bool,
    update_ids: bool,
    card_type: Option<String>,
    collection_path: PathBuf,
) -> Result<()> {
    use crate::inka::application::card_collector::CardCollector;

    info!(
        ?path,
        recursive, force, ignore_errors, full_sync, update_ids, ?card_type, "Collecting markdown cards"
    );

    // Initialize collector with force, full_sync, update_ids, ignore_errors, and card_type
    let mut collector = CardCollector::new(
        &collection_path,
        force,
        full_sync,
        update_ids,
        ignore_errors,
        card_type,
    )?;

    // Process based on path type
    let total_cards = if path.is_file() {
        // Single file
        collector.process_file(&path)?
    } else if path.is_dir() {
        if recursive {
            // Recursive directory processing
            collector.process_directory(&path)?
        } else {
            // Non-recursive - only process .md files in the directory
            let mut count = 0;
            for entry in std::fs::read_dir(&path)? {
                let entry = entry?;
                let entry_path = entry.path();
                if entry_path.is_file()
                    && entry_path.extension().and_then(|s| s.to_str()) == Some("md")
                {
                    count += collector.process_file(&entry_path)?;
                }
            }
            count
        }
    } else {
        return Err(anyhow::anyhow!("Path does not exist: {:?}", path));
    };

    // Print summary
    println!(
        "Successfully processed {} card{}",
        total_cards,
        if total_cards == 1 { "" } else { "s" }
    );

    // Print error summary if there were any errors
    let errors = collector.errors();
    if !errors.is_empty() {
        eprintln!(
            "\n{} error{} occurred:",
            errors.len(),
            if errors.len() == 1 { "" } else { "s" }
        );
        for error in errors {
            eprintln!("  {}", error);
        }
    }

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
