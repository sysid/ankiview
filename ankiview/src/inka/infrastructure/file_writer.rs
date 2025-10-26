use anyhow::{Context, Result};
use std::path::Path;

/// Read markdown file content
pub fn read_markdown_file(path: impl AsRef<Path>) -> Result<String> {
    std::fs::read_to_string(path.as_ref())
        .context("Failed to read markdown file")
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
}
