use anyhow::{Context, Result};
use sha2::{Sha256, Digest};
use std::path::Path;

/// Calculate SHA256 hash of a file's content
pub fn calculate_file_hash(path: impl AsRef<Path>) -> Result<String> {
    let content = std::fs::read_to_string(path.as_ref())
        .context("Failed to read file for hashing")?;

    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();

    // Convert to lowercase hex string
    Ok(format!("{:x}", result))
}

/// Check if file content has changed by comparing hashes
pub fn has_file_changed(path: impl AsRef<Path>, previous_hash: &str) -> Result<bool> {
    let current_hash = calculate_file_hash(path)?;
    Ok(current_hash != previous_hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn given_file_when_calculating_hash_then_returns_sha256() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");
        fs::write(&file_path, "Hello, world!").unwrap();

        let hash = calculate_file_hash(&file_path).unwrap();

        // SHA256 hash should be 64 hex characters
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn given_same_content_when_calculating_hash_then_returns_same_value() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.md");
        let file2 = temp_dir.path().join("file2.md");

        let content = "Identical content";
        fs::write(&file1, content).unwrap();
        fs::write(&file2, content).unwrap();

        let hash1 = calculate_file_hash(&file1).unwrap();
        let hash2 = calculate_file_hash(&file2).unwrap();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn given_different_content_when_calculating_hash_then_returns_different_values() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.md");
        let file2 = temp_dir.path().join("file2.md");

        fs::write(&file1, "Content A").unwrap();
        fs::write(&file2, "Content B").unwrap();

        let hash1 = calculate_file_hash(&file1).unwrap();
        let hash2 = calculate_file_hash(&file2).unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn given_nonexistent_file_when_calculating_hash_then_returns_error() {
        let result = calculate_file_hash("/nonexistent/file.md");

        assert!(result.is_err());
    }

    #[test]
    fn given_multiline_content_when_calculating_hash_then_handles_correctly() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("multi.md");

        let content = "Line 1\nLine 2\nLine 3\n";
        fs::write(&file_path, content).unwrap();

        let hash = calculate_file_hash(&file_path).unwrap();

        // Should produce valid SHA256 hash
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn given_matching_hash_when_checking_change_then_returns_false() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("unchanged.md");
        fs::write(&file_path, "Unchanged content").unwrap();

        let current_hash = calculate_file_hash(&file_path).unwrap();
        let changed = has_file_changed(&file_path, &current_hash).unwrap();

        assert!(!changed);
    }

    #[test]
    fn given_different_hash_when_checking_change_then_returns_true() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("changed.md");
        fs::write(&file_path, "New content").unwrap();

        let old_hash = "0000000000000000000000000000000000000000000000000000000000000000";
        let changed = has_file_changed(&file_path, old_hash).unwrap();

        assert!(changed);
    }

    #[test]
    fn given_file_modified_when_checking_then_detects_change() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("modified.md");

        // Write initial content and get hash
        fs::write(&file_path, "Original").unwrap();
        let original_hash = calculate_file_hash(&file_path).unwrap();

        // Modify content
        fs::write(&file_path, "Modified").unwrap();

        // Should detect change
        let changed = has_file_changed(&file_path, &original_hash).unwrap();
        assert!(changed);
    }
}
