// src/util/testing.rs

use anyhow::Result;
use std::collections::HashMap;
use std::env;
use tracing::{debug, info};
use tracing_subscriber::{
    filter::filter_fn,
    fmt::{self, format::FmtSpan},
    prelude::*,
    EnvFilter,
};

use crate::application::NoteRepository;
use crate::domain::{DomainError, Note};

// Common test environment variables
pub const TEST_ENV_VARS: &[&str] = &["RUST_LOG", "NO_CLEANUP"];

enum DeleteBehavior {
    Success(usize),
    NotFound,
}

/// Shared mock repository for testing use cases that depend on NoteRepository
///
/// This mock provides configurable behavior for all NoteRepository methods,
/// eliminating the need for each test file to define its own mock.
///
/// # Examples
///
/// ```
/// use ankiview::util::testing::MockNoteRepository;
/// use ankiview::domain::Note;
///
/// let mock = MockNoteRepository::builder()
///     .with_note(123, Note {
///         id: 123,
///         front: "Question".to_string(),
///         back: "Answer".to_string(),
///         tags: vec![],
///         model_name: "Basic".to_string(),
///     })
///     .with_delete_success(123, 2)
///     .build();
/// ```
pub struct MockNoteRepository {
    notes: HashMap<i64, Note>,
    delete_behaviors: HashMap<i64, DeleteBehavior>,
    search_results: HashMap<Option<String>, Vec<Note>>,
    notetypes: Vec<(i64, String)>,
}

impl MockNoteRepository {
    pub fn builder() -> MockNoteRepositoryBuilder {
        MockNoteRepositoryBuilder::new()
    }
}

impl NoteRepository for MockNoteRepository {
    fn get_note(&mut self, id: i64) -> Result<Note, DomainError> {
        self.notes
            .get(&id)
            .cloned()
            .ok_or(DomainError::NoteNotFound(id))
    }

    fn delete_note(&mut self, id: i64) -> Result<usize, DomainError> {
        match self.delete_behaviors.get(&id) {
            Some(DeleteBehavior::Success(count)) => Ok(*count),
            Some(DeleteBehavior::NotFound) => Err(DomainError::NoteNotFound(id)),
            None => Err(DomainError::NoteNotFound(id)),
        }
    }

    fn list_notes(&mut self, search_query: Option<&str>) -> Result<Vec<Note>, DomainError> {
        let key = search_query.map(|s| s.to_string());

        if let Some(results) = self.search_results.get(&key) {
            return Ok(results.clone());
        }

        // Default behavior: filter notes by front field if search query provided
        match search_query {
            None => Ok(self.notes.values().cloned().collect()),
            Some(query) => Ok(self
                .notes
                .values()
                .filter(|n| n.front.contains(query))
                .cloned()
                .collect()),
        }
    }

    fn list_notetypes(&mut self) -> Result<Vec<(i64, String)>, DomainError> {
        Ok(self.notetypes.clone())
    }
}

/// Builder for MockNoteRepository
///
/// Provides a fluent interface for configuring mock behavior.
pub struct MockNoteRepositoryBuilder {
    notes: HashMap<i64, Note>,
    delete_behaviors: HashMap<i64, DeleteBehavior>,
    search_results: HashMap<Option<String>, Vec<Note>>,
    notetypes: Vec<(i64, String)>,
}

impl MockNoteRepositoryBuilder {
    pub fn new() -> Self {
        Self {
            notes: HashMap::new(),
            delete_behaviors: HashMap::new(),
            search_results: HashMap::new(),
            notetypes: vec![],
        }
    }

    /// Add a note that can be retrieved by get_note
    pub fn with_note(mut self, id: i64, note: Note) -> Self {
        self.notes.insert(id, note);
        self
    }

    /// Configure delete_note to succeed for a specific ID
    pub fn with_delete_success(mut self, id: i64, deleted_cards: usize) -> Self {
        self.delete_behaviors
            .insert(id, DeleteBehavior::Success(deleted_cards));
        self
    }

    /// Configure delete_note to fail with NotFound for a specific ID
    pub fn with_delete_not_found(mut self, id: i64) -> Self {
        self.delete_behaviors
            .insert(id, DeleteBehavior::NotFound);
        self
    }

    /// Configure the result of list_notes for a specific search query
    ///
    /// # Arguments
    /// * `search_query` - None for all notes, Some(query) for filtered results
    /// * `results` - Notes to return for this query
    pub fn with_search_result(mut self, search_query: Option<String>, results: Vec<Note>) -> Self {
        self.search_results.insert(search_query, results);
        self
    }

    /// Add a notetype that can be listed
    pub fn with_notetype(mut self, id: i64, name: String) -> Self {
        self.notetypes.push((id, name));
        self
    }

    pub fn build(self) -> MockNoteRepository {
        MockNoteRepository {
            notes: self.notes,
            delete_behaviors: self.delete_behaviors,
            search_results: self.search_results,
            notetypes: self.notetypes,
        }
    }
}

impl Default for MockNoteRepositoryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub fn init_test_setup() -> Result<()> {
    // Set up logging first
    setup_test_logging();

    info!("Test Setup complete");
    Ok(())
}

fn setup_test_logging() {
    debug!("INIT: Attempting logger init from testing.rs");
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "trace");
    }

    // Create a filter for noisy modules
    let noisy_modules = ["skim", "html5ever", "reqwest", "mio"];
    let module_filter = filter_fn(move |metadata| {
        !noisy_modules
            .iter()
            .any(|name| metadata.target().starts_with(name))
    });

    // Set up the subscriber with environment filter
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"));

    // Build and set the subscriber
    let subscriber = tracing_subscriber::registry().with(
        fmt::layer()
            .with_writer(std::io::stderr)
            .with_target(true)
            .with_thread_names(false)
            .with_span_events(FmtSpan::CLOSE)
            .with_filter(module_filter)
            .with_filter(env_filter),
    );

    // Only set if we haven't already set a global subscriber
    if tracing::dispatcher::has_been_set() {
        debug!("Tracing subscriber already set");
    } else {
        subscriber.try_init().unwrap_or_else(|e| {
            eprintln!("Error: Failed to set up logging: {}", e);
        });
    }
}

pub fn print_active_env_vars() {
    for var in TEST_ENV_VARS {
        if let Ok(value) = env::var(var) {
            println!("{var}={value}");
        } else {
            println!("{var} is not set");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ctor::ctor]
    fn init() {
        init_test_setup().expect("Failed to initialize test setup");
    }

    #[test]
    fn given_note_added_when_getting_note_then_returns_note() {
        let test_note = Note {
            id: 123,
            front: "Test Question".to_string(),
            back: "Test Answer".to_string(),
            tags: vec!["tag1".to_string()],
            model_name: "Basic".to_string(),
        };

        let mut mock = MockNoteRepository::builder()
            .with_note(123, test_note.clone())
            .build();

        let result = mock.get_note(123).expect("Note should exist");
        assert_eq!(result.id, 123);
        assert_eq!(result.front, "Test Question");
    }

    #[test]
    fn given_no_note_when_getting_note_then_returns_error() {
        let mut mock = MockNoteRepository::builder().build();

        let result = mock.get_note(999);
        assert!(result.is_err());
        assert!(matches!(result, Err(DomainError::NoteNotFound(999))));
    }

    #[test]
    fn given_delete_success_configured_when_deleting_then_returns_card_count() {
        let mut mock = MockNoteRepository::builder()
            .with_delete_success(123, 2)
            .build();

        let result = mock.delete_note(123).expect("Delete should succeed");
        assert_eq!(result, 2);
    }

    #[test]
    fn given_delete_not_found_configured_when_deleting_then_returns_error() {
        let mut mock = MockNoteRepository::builder()
            .with_delete_not_found(123)
            .build();

        let result = mock.delete_note(123);
        assert!(result.is_err());
        assert!(matches!(result, Err(DomainError::NoteNotFound(123))));
    }

    #[test]
    fn given_multiple_notes_when_listing_all_then_returns_all_notes() {
        let note1 = Note {
            id: 1,
            front: "Question 1".to_string(),
            back: "Answer 1".to_string(),
            tags: vec![],
            model_name: "Basic".to_string(),
        };
        let note2 = Note {
            id: 2,
            front: "Question 2".to_string(),
            back: "Answer 2".to_string(),
            tags: vec![],
            model_name: "Basic".to_string(),
        };

        let mut mock = MockNoteRepository::builder()
            .with_note(1, note1)
            .with_note(2, note2)
            .build();

        let result = mock.list_notes(None).expect("List should succeed");
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn given_search_query_when_listing_notes_then_filters_by_front_field() {
        let note1 = Note {
            id: 1,
            front: "What is a Tree?".to_string(),
            back: "Answer 1".to_string(),
            tags: vec![],
            model_name: "Basic".to_string(),
        };
        let note2 = Note {
            id: 2,
            front: "What is a Graph?".to_string(),
            back: "Answer 2".to_string(),
            tags: vec![],
            model_name: "Basic".to_string(),
        };

        let mut mock = MockNoteRepository::builder()
            .with_note(1, note1)
            .with_note(2, note2)
            .build();

        let result = mock.list_notes(Some("Tree")).expect("List should succeed");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, 1);
    }

    #[test]
    fn given_custom_search_result_when_listing_then_returns_configured_result() {
        let custom_result = vec![Note {
            id: 999,
            front: "Custom".to_string(),
            back: "Result".to_string(),
            tags: vec![],
            model_name: "Basic".to_string(),
        }];

        let mut mock = MockNoteRepository::builder()
            .with_search_result(Some("custom".to_string()), custom_result.clone())
            .build();

        let result = mock.list_notes(Some("custom")).expect("List should succeed");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, 999);
    }

    #[test]
    fn given_notetypes_added_when_listing_then_returns_notetypes() {
        let mut mock = MockNoteRepository::builder()
            .with_notetype(1, "Basic".to_string())
            .with_notetype(2, "Cloze".to_string())
            .build();

        let result = mock.list_notetypes().expect("List should succeed");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].1, "Basic");
        assert_eq!(result[1].1, "Cloze");
    }
}
