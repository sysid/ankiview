use anyhow::{Context, Result};
use sha2::{Sha256, Digest};
use std::path::Path;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

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

/// Hash cache for tracking file changes
/// Stores filepath -> hash mapping in a JSON file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashCache {
    cache_path: std::path::PathBuf,
    hashes: HashMap<String, String>,
}

impl HashCache {
    /// Load hash cache from file, or create empty cache if file doesn't exist
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let cache_path = path.as_ref().to_path_buf();

        let hashes = if cache_path.exists() {
            let content = std::fs::read_to_string(&cache_path)
                .context("Failed to read hash cache file")?;
            serde_json::from_str(&content)
                .context("Failed to parse hash cache JSON")?
        } else {
            HashMap::new()
        };

        Ok(Self { cache_path, hashes })
    }

    /// Save hash cache to file
    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.hashes)
            .context("Failed to serialize hash cache")?;

        std::fs::write(&self.cache_path, json)
            .context("Failed to write hash cache file")?;

        Ok(())
    }

    /// Check if file has changed compared to cached hash
    /// Returns true if file is new or content has changed
    pub fn file_has_changed(&self, filepath: impl AsRef<Path>) -> Result<bool> {
        let path_str = filepath.as_ref()
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid file path"))?
            .to_string();

        // If not in cache, it's a new file (changed)
        let Some(cached_hash) = self.hashes.get(&path_str) else {
            return Ok(true);
        };

        // Compare current hash with cached hash
        has_file_changed(filepath, cached_hash)
    }

    /// Update hash for a file in the cache
    pub fn update_hash(&mut self, filepath: impl AsRef<Path>) -> Result<()> {
        let path_str = filepath.as_ref()
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid file path"))?
            .to_string();

        let hash = calculate_file_hash(filepath)?;
        self.hashes.insert(path_str, hash);

        Ok(())
    }

    /// Clear all hashes from cache
    pub fn clear(&mut self) {
        self.hashes.clear();
    }
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

    // HashCache tests
    #[test]
    fn given_nonexistent_cache_when_loading_then_creates_empty() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("hashes.json");

        let cache = HashCache::load(&cache_path).unwrap();

        assert_eq!(cache.hashes.len(), 0);
    }

    #[test]
    fn given_cache_when_saving_then_creates_json_file() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");

        let cache = HashCache::load(&cache_path).unwrap();
        cache.save().unwrap();

        assert!(cache_path.exists());
        let content = fs::read_to_string(&cache_path).unwrap();
        assert!(content.contains("{") && content.contains("}"));
    }

    #[test]
    fn given_new_file_when_checking_then_returns_changed() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let file_path = temp_dir.path().join("test.md");
        fs::write(&file_path, "Content").unwrap();

        let cache = HashCache::load(&cache_path).unwrap();
        let changed = cache.file_has_changed(&file_path).unwrap();

        assert!(changed);
    }

    #[test]
    fn given_unchanged_file_when_checking_then_returns_false() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let file_path = temp_dir.path().join("unchanged.md");
        fs::write(&file_path, "Stable content").unwrap();

        let mut cache = HashCache::load(&cache_path).unwrap();
        cache.update_hash(&file_path).unwrap();
        cache.save().unwrap();

        // Reload cache and check same file
        let cache = HashCache::load(&cache_path).unwrap();
        let changed = cache.file_has_changed(&file_path).unwrap();

        assert!(!changed);
    }

    #[test]
    fn given_modified_file_when_checking_then_returns_changed() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let file_path = temp_dir.path().join("modified.md");
        fs::write(&file_path, "Original").unwrap();

        let mut cache = HashCache::load(&cache_path).unwrap();
        cache.update_hash(&file_path).unwrap();
        cache.save().unwrap();

        // Modify file
        fs::write(&file_path, "Modified").unwrap();

        // Reload and check
        let cache = HashCache::load(&cache_path).unwrap();
        let changed = cache.file_has_changed(&file_path).unwrap();

        assert!(changed);
    }

    #[test]
    fn given_cache_with_hashes_when_clearing_then_removes_all() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let file_path = temp_dir.path().join("file.md");
        fs::write(&file_path, "Content").unwrap();

        let mut cache = HashCache::load(&cache_path).unwrap();
        cache.update_hash(&file_path).unwrap();
        assert_eq!(cache.hashes.len(), 1);

        cache.clear();
        assert_eq!(cache.hashes.len(), 0);
    }

    #[test]
    fn given_multiple_files_when_updating_then_tracks_all() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let file1 = temp_dir.path().join("file1.md");
        let file2 = temp_dir.path().join("file2.md");
        fs::write(&file1, "Content 1").unwrap();
        fs::write(&file2, "Content 2").unwrap();

        let mut cache = HashCache::load(&cache_path).unwrap();
        cache.update_hash(&file1).unwrap();
        cache.update_hash(&file2).unwrap();
        cache.save().unwrap();

        // Reload and verify both tracked
        let cache = HashCache::load(&cache_path).unwrap();
        assert_eq!(cache.hashes.len(), 2);
        assert!(!cache.file_has_changed(&file1).unwrap());
        assert!(!cache.file_has_changed(&file2).unwrap());
    }
}
