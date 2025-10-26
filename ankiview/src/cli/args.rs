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
    Collect {
        /// Path to markdown file or directory
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Process directory recursively
        #[arg(short, long)]
        recursive: bool,
    },
}
