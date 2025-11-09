// src/constants.rs
//
// Application-wide constants extracted from magic numbers throughout the codebase.
// Each constant is documented with its purpose and usage context.

/// Characters to search backward from note pattern when looking for existing ID comment.
///
/// When injecting an ID into markdown, we check the last N characters before the note
/// to see if an ID comment already exists. This prevents duplicate IDs.
///
/// Used in: `inka/infrastructure/file_writer.rs`
pub const ID_SEARCH_RANGE_BEFORE: usize = 50;

/// Characters to search forward from ID comment when replacing note IDs.
///
/// When replacing an existing ID, we search this many characters after the ID comment
/// to find the associated note pattern.
///
/// Used in: `inka/infrastructure/file_writer.rs`
pub const ID_SEARCH_RANGE_AFTER: usize = 100;

/// Delay in milliseconds after writing HTML file before opening browser.
///
/// On macOS, the browser needs a brief moment for the file to be fully written
/// and indexed before opening. Without this delay, the browser may open an empty
/// or incomplete file.
///
/// Used in: `infrastructure/renderer.rs`
pub const BROWSER_LAUNCH_DELAY_MS: u64 = 500;
