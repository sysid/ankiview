use ankiview::domain::Note;
use anyhow::Result;

#[test]
fn given_note_when_serializing_to_json_then_contains_all_fields() -> Result<()> {
    // Arrange
    let note = Note {
        id: 1234567890,
        front: "Test front".to_string(),
        back: "Test back".to_string(),
        tags: vec!["tag1".to_string(), "tag2".to_string()],
        model_name: "Basic".to_string(),
    };

    // Act
    let json = serde_json::to_string_pretty(&note)?;

    // Assert
    assert!(json.contains(r#""id": 1234567890"#));
    assert!(json.contains(r#""front": "Test front""#));
    assert!(json.contains(r#""back": "Test back""#));
    assert!(json.contains(r#""tags": ["#));
    assert!(json.contains(r#""tag1""#));
    assert!(json.contains(r#""tag2""#));
    assert!(json.contains(r#""model_name": "Basic""#));
    Ok(())
}

#[test]
fn given_note_when_serializing_then_uses_snake_case_fields() -> Result<()> {
    // Arrange
    let note = Note {
        id: 123,
        front: "F".to_string(),
        back: "B".to_string(),
        tags: vec![],
        model_name: "Model".to_string(),
    };

    // Act
    let json = serde_json::to_string(&note)?;

    // Assert - field names should be snake_case, not camelCase
    assert!(json.contains(r#""model_name""#));
    assert!(!json.contains(r#""modelName""#));
    Ok(())
}

#[test]
fn given_note_with_empty_tags_when_serializing_then_produces_empty_array() -> Result<()> {
    // Arrange
    let note = Note {
        id: 123,
        front: "F".to_string(),
        back: "B".to_string(),
        tags: vec![],
        model_name: "Model".to_string(),
    };

    // Act
    let json = serde_json::to_string_pretty(&note)?;

    // Assert
    assert!(json.contains(r#""tags": []"#));
    Ok(())
}
