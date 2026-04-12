// tests/test_note_updater.rs — US2 tests for NoteUpdater use case
use ankiview::application::NoteUpdater;
use ankiview::domain::Note;
use ankiview::util::testing::MockNoteRepository;

fn note_with_tags(id: i64, tags: Vec<String>) -> Note {
    Note {
        id,
        front: "Q".to_string(),
        back: "A".to_string(),
        tags,
        model_name: "Basic".to_string(),
    }
}

// T021: tag add adds tag to existing note
#[test]
fn given_existing_note_when_adding_tag_then_tag_is_added() {
    let note = note_with_tags(123, vec!["physics".to_string()]);
    let repo = MockNoteRepository::builder().with_note(123, note).build();
    let mut updater = NoteUpdater::new(repo);

    updater.add_tags(123, &["review".to_string()]).unwrap();
}

// T022: tag remove removes tag from existing note
#[test]
fn given_existing_note_when_removing_tag_then_tag_is_removed() {
    let note = note_with_tags(123, vec!["physics".to_string(), "review".to_string()]);
    let repo = MockNoteRepository::builder().with_note(123, note).build();
    let mut updater = NoteUpdater::new(repo);

    updater.remove_tags(123, &["review".to_string()]).unwrap();
}

// T023: tag add on nonexistent note returns error
#[test]
fn given_nonexistent_note_when_adding_tag_then_error() {
    let repo = MockNoteRepository::builder().build();
    let mut updater = NoteUpdater::new(repo);

    let result = updater.add_tags(99999, &["review".to_string()]);
    assert!(result.is_err());
}

// T024: tag add with hierarchical tag works
#[test]
fn given_existing_note_when_adding_hierarchical_tag_then_stored() {
    let note = note_with_tags(123, vec![]);
    let repo = MockNoteRepository::builder().with_note(123, note).build();
    let mut updater = NoteUpdater::new(repo);

    updater
        .add_tags(123, &["topic::math::algebra".to_string()])
        .unwrap();
}
