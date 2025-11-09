// src/application/note_lister.rs
use crate::application::NoteRepository;
use crate::domain::{DomainError, Note};

pub struct NoteLister<R: NoteRepository> {
    repository: R,
}

impl<R: NoteRepository> NoteLister<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// List all notes, or filter by search query
    ///
    /// # Arguments
    /// * `search_query` - Optional search term to filter front field
    ///
    /// # Returns
    /// Vector of notes matching the criteria
    pub fn list_notes(&mut self, search_query: Option<&str>) -> Result<Vec<Note>, DomainError> {
        self.repository.list_notes(search_query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Note;

    struct MockRepository {
        notes: Vec<Note>,
    }

    impl NoteRepository for MockRepository {
        fn get_note(&mut self, id: i64) -> Result<Note, DomainError> {
            self.notes
                .iter()
                .find(|n| n.id == id)
                .cloned()
                .ok_or(DomainError::NoteNotFound(id))
        }

        fn delete_note(&mut self, _id: i64) -> Result<usize, DomainError> {
            unimplemented!()
        }

        fn list_notes(&mut self, search_query: Option<&str>) -> Result<Vec<Note>, DomainError> {
            Ok(match search_query {
                None => self.notes.clone(),
                Some(query) => self
                    .notes
                    .iter()
                    .filter(|n| n.front.contains(query))
                    .cloned()
                    .collect(),
            })
        }

        fn list_notetypes(&mut self) -> Result<Vec<(i64, String)>, DomainError> {
            unimplemented!()
        }
    }

    #[test]
    fn given_no_search_when_listing_notes_then_returns_all_notes() {
        // Arrange
        let notes = vec![
            Note {
                id: 1,
                front: "First".to_string(),
                back: "Back1".to_string(),
                tags: vec![],
                model_name: "Basic".to_string(),
            },
            Note {
                id: 2,
                front: "Second".to_string(),
                back: "Back2".to_string(),
                tags: vec![],
                model_name: "Basic".to_string(),
            },
        ];
        let repo = MockRepository {
            notes: notes.clone(),
        };
        let mut lister = NoteLister::new(repo);

        // Act
        let result = lister.list_notes(None).unwrap();

        // Assert
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn given_search_query_when_listing_notes_then_returns_filtered_notes() {
        // Arrange
        let notes = vec![
            Note {
                id: 1,
                front: "What is a Tree?".to_string(),
                back: "Back1".to_string(),
                tags: vec![],
                model_name: "Basic".to_string(),
            },
            Note {
                id: 2,
                front: "What is a Graph?".to_string(),
                back: "Back2".to_string(),
                tags: vec![],
                model_name: "Basic".to_string(),
            },
        ];
        let repo = MockRepository {
            notes: notes.clone(),
        };
        let mut lister = NoteLister::new(repo);

        // Act
        let result = lister.list_notes(Some("Tree")).unwrap();

        // Assert
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, 1);
    }
}
