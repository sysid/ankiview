use anyhow::{Context, Result};
use std::path::Path;

/// Strip ID comment lines from note string
/// Returns the note text without any <!--ID:...--> lines
pub fn strip_id_comment(note_str: &str) -> String {
    note_str
        .lines()
        .filter(|line| !line.trim().starts_with("<!--ID:"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Read markdown file content
pub fn read_markdown_file(path: impl AsRef<Path>) -> Result<String> {
    std::fs::read_to_string(path.as_ref()).context("Failed to read markdown file")
}

/// Write markdown content to file
pub fn write_markdown_file(path: impl AsRef<Path>, content: &str) -> Result<()> {
    std::fs::write(path.as_ref(), content).context("Failed to write markdown file")
}

/// Inject Anki ID before a note in markdown content
/// If the note already has an ID, returns content unchanged
pub fn inject_anki_id(content: &str, note_pattern: &str, anki_id: i64) -> String {
    // Find the position of the note pattern
    let Some(note_pos) = content.find(note_pattern) else {
        // Pattern not found, return unchanged
        return content.to_string();
    };

    // Check if there's already an ID before this note
    // Look at the content before the note pattern
    let before_note = &content[..note_pos];

    // Check if the previous line (or within a few chars) has an ID comment
    // We'll look for <!--ID: pattern in the last 50 chars before the note
    let check_start = before_note.len().saturating_sub(50);
    let check_region = &before_note[check_start..];

    if check_region.contains("<!--ID:") {
        // ID already exists, return unchanged
        return content.to_string();
    }

    // No ID exists, inject one before the note pattern
    let id_comment = format!("<!--ID:{}-->\n", anki_id);
    let mut result = String::with_capacity(content.len() + id_comment.len());
    result.push_str(&content[..note_pos]);
    result.push_str(&id_comment);
    result.push_str(&content[note_pos..]);

    result
}

/// Replace an existing Anki ID with a new one for a specific note
/// If no ID exists before the note, injects a new one
pub fn replace_anki_id(content: &str, note_pattern: &str, new_id: i64) -> String {
    // Find the position of the note pattern
    let Some(note_pos) = content.find(note_pattern) else {
        // Pattern not found, return unchanged
        return content.to_string();
    };

    // Check if there's already an ID before this note
    let before_note = &content[..note_pos];

    // Look for <!--ID: pattern in the last 100 chars before the note
    // Use rfind to get the LAST occurrence (closest to the note)
    let check_start = before_note.len().saturating_sub(100);
    let check_region = &before_note[check_start..];

    if let Some(id_start_rel) = check_region.rfind("<!--ID:") {
        // Found an ID comment, extract it
        let id_start = check_start + id_start_rel;

        // Find the end of the ID comment
        if let Some(id_end_rel) = check_region[id_start_rel..].find("-->") {
            let id_end = check_start + id_start_rel + id_end_rel + 3; // +3 for "-->".len()

            // Check if there's a newline right after the ID comment
            let after_id_newline = if content[id_end..].starts_with('\n') {
                1
            } else {
                0
            };

            // Replace the old ID comment with the new one, preserving newline
            let new_id_comment = format!("<!--ID:{}-->\n", new_id);
            let mut result = String::with_capacity(content.len());
            result.push_str(&content[..id_start]);
            result.push_str(&new_id_comment);
            result.push_str(&content[id_end + after_id_newline..]);

            return result;
        }
    }

    // No ID exists, inject one before the note pattern (same as inject_anki_id)
    let id_comment = format!("<!--ID:{}-->\n", new_id);
    let mut result = String::with_capacity(content.len() + id_comment.len());
    result.push_str(&content[..note_pos]);
    result.push_str(&id_comment);
    result.push_str(&content[note_pos..]);

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn given_markdown_file_when_reading_then_returns_content() {
        // Create temp file
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");

        let content = "# Test\n\nSome content";
        fs::write(&file_path, content).unwrap();

        // Read file
        let result = read_markdown_file(&file_path).unwrap();

        assert_eq!(result, content);
    }

    #[test]
    fn given_file_with_ids_when_reading_then_preserves_ids() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");

        let content = r#"---
Deck: Test

<!--ID:1234567890-->
1. Question?
> Answer!
---"#;
        fs::write(&file_path, content).unwrap();

        let result = read_markdown_file(&file_path).unwrap();

        assert!(result.contains("<!--ID:1234567890-->"));
        assert_eq!(result, content);
    }

    #[test]
    fn given_nonexistent_file_when_reading_then_returns_error() {
        let result = read_markdown_file("/nonexistent/path/file.md");

        assert!(result.is_err());
    }

    #[test]
    fn given_note_without_id_when_injecting_then_adds_id() {
        let content = r#"---
Deck: Test

1. Question?
> Answer!
---"#;

        let result = inject_anki_id(content, "1. Question?", 1234567890);

        assert!(result.contains("<!--ID:1234567890-->"));
        assert!(result.contains("<!--ID:1234567890-->\n1. Question?"));
    }

    #[test]
    fn given_note_with_existing_id_when_injecting_then_unchanged() {
        let content = r#"---
Deck: Test

<!--ID:9999999999-->
1. Question?
> Answer!
---"#;

        let result = inject_anki_id(content, "1. Question?", 1234567890);

        // Should keep original ID
        assert!(result.contains("<!--ID:9999999999-->"));
        assert!(!result.contains("<!--ID:1234567890-->"));
        assert_eq!(result, content);
    }

    #[test]
    fn given_multiple_notes_when_injecting_then_targets_correct_note() {
        let content = r#"---
Deck: Test

1. First question?
> First answer

2. Second question?
> Second answer
---"#;

        let result = inject_anki_id(content, "2. Second question?", 5555555555);

        assert!(result.contains("<!--ID:5555555555-->\n2. Second question?"));
        // First note should remain untouched
        assert!(result.contains("1. First question?\n> First answer"));
    }

    #[test]
    fn given_note_pattern_when_injecting_then_preserves_formatting() {
        let content = "Some text\n\n1. Question\n> Answer\n\nMore text";

        let result = inject_anki_id(content, "1. Question", 1111111111);

        assert_eq!(
            result,
            "Some text\n\n<!--ID:1111111111-->\n1. Question\n> Answer\n\nMore text"
        );
    }

    #[test]
    fn given_content_when_writing_then_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("output.md");

        let content = "# Test\n\nSome content";

        write_markdown_file(&file_path, content).unwrap();

        assert!(file_path.exists());
        let written = fs::read_to_string(&file_path).unwrap();
        assert_eq!(written, content);
    }

    #[test]
    fn given_existing_file_when_writing_then_overwrites() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("output.md");

        // Write initial content
        fs::write(&file_path, "Old content").unwrap();

        // Overwrite with new content
        let new_content = "New content";
        write_markdown_file(&file_path, new_content).unwrap();

        let written = fs::read_to_string(&file_path).unwrap();
        assert_eq!(written, new_content);
    }

    #[test]
    fn given_round_trip_when_reading_and_writing_then_preserves_content() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("roundtrip.md");

        let original = r#"---
Deck: Test

<!--ID:1234567890-->
1. Question?
> Answer!

2. Another question
> Another answer
---"#;

        // Write
        write_markdown_file(&file_path, original).unwrap();

        // Read back
        let read_back = read_markdown_file(&file_path).unwrap();

        assert_eq!(read_back, original);
    }

    #[test]
    fn given_note_with_existing_id_when_replacing_then_updates_id() {
        let content = r#"---
Deck: Test

<!--ID:1111111111-->
1. Question?
> Answer!
---"#;

        let result = replace_anki_id(content, "1. Question?", 9999999999);

        // Should have new ID
        assert!(result.contains("<!--ID:9999999999-->"));
        // Should NOT have old ID
        assert!(!result.contains("<!--ID:1111111111-->"));
        // Should have only one ID comment
        assert_eq!(result.matches("<!--ID:").count(), 1);
    }

    #[test]
    fn given_note_without_id_when_replacing_then_injects_id() {
        let content = r#"---
Deck: Test

1. Question?
> Answer!
---"#;

        let result = replace_anki_id(content, "1. Question?", 5555555555);

        // Should have new ID
        assert!(result.contains("<!--ID:5555555555-->"));
        assert_eq!(result.matches("<!--ID:").count(), 1);
    }

    #[test]
    fn given_multiple_notes_with_ids_when_replacing_then_targets_correct_note() {
        let content = r#"---
Deck: Test

<!--ID:1111111111-->
1. First question?
> First answer

<!--ID:2222222222-->
2. Second question?
> Second answer
---"#;

        let result = replace_anki_id(content, "2. Second question?", 9999999999);

        // First ID should remain unchanged
        assert!(result.contains("<!--ID:1111111111-->"));
        // Second ID should be replaced
        assert!(result.contains("<!--ID:9999999999-->"));
        assert!(!result.contains("<!--ID:2222222222-->"));
        // Should have exactly two ID comments
        assert_eq!(result.matches("<!--ID:").count(), 2);
    }
}
