// src/application/mod.rs
pub mod note_deleter;
pub mod note_viewer;
pub mod profile;

pub use note_deleter::NoteDeleter;
pub use note_viewer::{NoteRepository, NoteViewer};
pub use profile::ProfileLocator;
