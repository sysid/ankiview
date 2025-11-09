// src/args.rs
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)] // Read from `Cargo.toml`
#[command(arg_required_else_help = true, disable_help_subcommand = true)]
pub struct Args {
    /// Path to Anki collection file (optional)
    #[arg(short, long, value_name = "COLLECTION", global = true)]
    pub collection: Option<PathBuf>,

    /// Profile name (optional)
    #[arg(short, long, value_name = "PROFILE", global = true)]
    pub profile: Option<String>,

    /// Verbosity level (-v = debug, -vv = trace)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Subcommand to execute (view, delete, or list)
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// View a note in the browser
    View {
        /// Note ID to view
        #[arg(value_name = "NOTE_ID")]
        note_id: i64,

        /// Output note as JSON instead of opening in browser
        #[arg(long)]
        json: bool,
    },

    /// Delete a note from the collection
    Delete {
        /// Note ID to delete
        #[arg(value_name = "NOTE_ID")]
        note_id: i64,
    },

    /// List notes with ID and first line of front field
    List {
        /// Optional search term to filter notes by front field content
        #[arg(value_name = "SEARCH")]
        search: Option<String>,
    },

    /// Collect markdown cards into Anki
    ///
    /// Processes markdown files containing flashcards and imports them into your Anki collection.
    /// Cards are automatically tracked with ID comments, allowing updates without creating duplicates.
    Collect {
        /// Path to markdown file or directory containing .md files
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Process directory recursively, scanning all subdirectories for .md files.
        /// Without this flag, only processes files in the specified directory (non-recursive).
        #[arg(short, long)]
        recursive: bool,

        /// Overwrite media files when filename conflicts occur in collection.media/.
        /// Without this flag, processing stops with an error if a different file with the same name exists.
        /// Use when you want to replace existing images with updated versions.
        #[arg(long)]
        force: bool,

        /// Continue processing remaining files even if errors occur.
        /// Errors are collected and reported at the end instead of stopping immediately.
        /// Useful for batch processing where you want to see all issues at once.
        #[arg(short, long)]
        ignore_errors: bool,

        /// Process all files regardless of hash cache, forcing a complete rebuild.
        /// By default, unchanged files are skipped for performance (tracked via SHA256 hashes).
        /// Use this when you want to ensure all cards are re-processed from scratch.
        #[arg(short = 'f', long)]
        full_sync: bool,

        /// Search Anki for existing notes by content and inject their IDs into markdown.
        /// Prevents duplicate creation when markdown files lack ID comments (<!--ID:123-->).
        /// Useful for recovering lost IDs or importing cards from other sources.
        /// Matches notes by comparing HTML field content.
        #[arg(short = 'u', long)]
        update_ids: bool,

        /// Card type (notetype) to use when creating notes.
        /// Specify exact notetype name (e.g., "Basic", "Inka Basic").
        /// Defaults to "Inka Basic" if not specified.
        /// Use 'list-card-types' command to see available card types.
        #[arg(long, value_name = "TYPE")]
        card_type: Option<String>,
    },

    /// List available card types (notetypes) in the collection
    ///
    /// Displays all available note types that can be used with the --card-type flag.
    /// Each card type defines the fields and card templates for flashcards.
    ListCardTypes,
}
