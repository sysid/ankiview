// tests/test_tag_manager.rs — US4 tests for TagManager use case
use ankiview::application::TagManager;
use ankiview::domain::Note;
use ankiview::util::testing::MockNoteRepository;

fn note_with_tags(id: i64, tags: Vec<String>) -> Note {
    Note {
        id,
        front: format!("Q{}", id),
        back: format!("A{}", id),
        tags,
        model_name: "Basic".to_string(),
    }
}

// T049: rename mode
#[test]
fn given_notes_with_tag_when_replacing_then_renamed() {
    let repo = MockNoteRepository::builder()
        .with_note(100, note_with_tags(100, vec!["review".to_string()]))
        .with_note(200, note_with_tags(200, vec!["review".to_string()]))
        .build();
    let mut manager = TagManager::new(repo);

    let affected = manager.replace_tag(None, "review", "reviewed").unwrap();
    assert_eq!(affected, 2);
}

// T050: bulk add mode
#[test]
fn given_notes_when_bulk_adding_then_all_get_tag() {
    let repo = MockNoteRepository::builder()
        .with_note(100, note_with_tags(100, vec![]))
        .with_note(200, note_with_tags(200, vec![]))
        .build();
    let mut manager = TagManager::new(repo);

    let affected = manager.replace_tag(None, "", "batch-2026").unwrap();
    assert_eq!(affected, 2);
}

// T051: bulk remove mode
#[test]
fn given_notes_with_tag_when_bulk_removing_then_tag_removed() {
    let repo = MockNoteRepository::builder()
        .with_note(100, note_with_tags(100, vec!["obsolete".to_string()]))
        .with_note(200, note_with_tags(200, vec![]))
        .build();
    let mut manager = TagManager::new(repo);

    let affected = manager.replace_tag(None, "obsolete", "").unwrap();
    assert_eq!(affected, 1);
}

// T053: both empty returns validation error
#[test]
fn given_both_empty_when_replacing_then_error() {
    let repo = MockNoteRepository::builder().build();
    let mut manager = TagManager::new(repo);

    let result = manager.replace_tag(None, "", "");
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("cannot be empty"));
}

// T054: reports correct affected count
#[test]
fn given_three_notes_one_matching_when_replacing_then_reports_one() {
    let repo = MockNoteRepository::builder()
        .with_note(100, note_with_tags(100, vec!["target".to_string()]))
        .with_note(200, note_with_tags(200, vec!["other".to_string()]))
        .with_note(300, note_with_tags(300, vec!["other".to_string()]))
        .build();
    let mut manager = TagManager::new(repo);

    let affected = manager.replace_tag(None, "target", "replaced").unwrap();
    assert_eq!(affected, 1);
}
