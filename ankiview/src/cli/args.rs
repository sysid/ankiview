// src/args.rs
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)] // Read from `Cargo.toml`
#[command(args_conflicts_with_subcommands = true)]
#[command(arg_required_else_help = true, disable_help_subcommand = true)]
pub struct Args {
    /// Optional subcommand (defaults to 'view' for backward compatibility)
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Note ID (used when no subcommand is provided - backward compatibility)
    #[arg(value_name = "NOTE_ID")]
    pub note_id: Option<i64>,

    /// Path to Anki collection file (optional)
    #[arg(short, long, value_name = "COLLECTION")]
    pub collection: Option<PathBuf>,

    /// Profile name (optional)
    #[arg(short, long, value_name = "PROFILE")]
    pub profile: Option<String>,

    /// Verbosity level (-v = debug, -vv = trace)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// View a note in the browser (default action)
    View {
        /// Note ID to view
        #[arg(value_name = "NOTE_ID")]
        note_id: i64,

        /// Path to Anki collection file (optional)
        #[arg(short, long, value_name = "COLLECTION")]
        collection: Option<PathBuf>,

        /// Profile name (optional)
        #[arg(short, long, value_name = "PROFILE")]
        profile: Option<String>,
    },

    /// Delete a note from the collection
    Delete {
        /// Note ID to delete
        #[arg(value_name = "NOTE_ID")]
        note_id: i64,

        /// Path to Anki collection file (optional)
        #[arg(short, long, value_name = "COLLECTION")]
        collection: Option<PathBuf>,

        /// Profile name (optional)
        #[arg(short, long, value_name = "PROFILE")]
        profile: Option<String>,
    },
}

impl Args {
    /// Resolve the command, defaulting to View for backward compatibility
    pub fn resolve_command(&self) -> Command {
        match &self.command {
            Some(cmd) => cmd.clone(),
            None => {
                // Backward compatibility: treat bare note_id as View command
                // Flags from top-level will be merged in run()
                Command::View {
                    note_id: self.note_id.expect("note_id required when no subcommand"),
                    collection: None,
                    profile: None,
                }
            }
        }
    }
}
