// src/application/tag_manager.rs
use crate::application::NoteRepository;
use crate::domain::DomainError;

pub struct TagManager<R: NoteRepository> {
    repository: R,
}

impl<R: NoteRepository> TagManager<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn replace_tag(
        &mut self,
        query: Option<&str>,
        old_tag: &str,
        new_tag: &str,
    ) -> Result<usize, DomainError> {
        if old_tag.is_empty() && new_tag.is_empty() {
            return Err(DomainError::CollectionError(
                "Both --old and --new cannot be empty".to_string(),
            ));
        }
        self.repository.replace_tag(query, old_tag, new_tag)
    }
}
