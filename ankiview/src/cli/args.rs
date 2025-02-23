// src/args.rs
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)] // Read from `Cargo.toml`
#[command(arg_required_else_help = true, disable_help_subcommand = true)]
pub struct Args {
    /// The Anki note ID to view
    #[arg(value_name = "NOTE_ID")]
    pub note_id: i64,

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
