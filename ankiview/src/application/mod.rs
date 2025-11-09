// src/application/mod.rs
pub mod note_deleter;
pub mod note_lister;
pub mod note_viewer;

pub use note_deleter::NoteDeleter;
pub use note_lister::NoteLister;
pub use note_viewer::{NoteRepository, NoteViewer};
