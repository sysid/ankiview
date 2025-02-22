use ankiview::cli::args::Args;
// src/main.rs
use anyhow::Result;
use clap::Parser;
use tracing::Level;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging based on verbosity
    let filter = match args.verbose {
        0 => Level::INFO,
        1 => Level::DEBUG,
        _ => Level::TRACE,
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive(format!("ankiview={}", filter).parse().unwrap()),
        )
        .init();

    ankiview::run(args)
}
