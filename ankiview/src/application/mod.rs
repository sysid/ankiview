// src/application/mod.rs
pub mod note_deleter;
pub mod note_editor;
pub mod note_lister;
pub mod note_updater;
pub mod note_viewer;
pub mod tag_manager;

pub use note_deleter::NoteDeleter;
pub use note_editor::NoteEditor;
pub use note_lister::NoteLister;
pub use note_updater::NoteUpdater;
pub use note_viewer::{NoteRepository, NoteViewer};
pub use tag_manager::TagManager;
