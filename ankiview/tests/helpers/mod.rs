use ankiview::infrastructure::AnkiRepository;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Test fixture for working with temporary Anki collections
#[allow(dead_code)]
pub struct TestCollection {
    _temp_dir: TempDir,
    pub collection_path: PathBuf,
    pub media_dir: PathBuf,
}

impl TestCollection {
    /// Create a new test collection by copying the fixture
    pub fn new() -> Result<Self> {
        let temp_dir = tempfile::tempdir()
            .context("Failed to create temporary directory")?;

        let fixture_path = Self::fixture_collection_path();
        let collection_path = temp_dir.path().join("collection.anki2");

        // Copy fixture collection to temp location
        std::fs::copy(&fixture_path, &collection_path)
            .context("Failed to copy test collection fixture")?;

        // Copy media directory
        let fixture_media = fixture_path.parent().unwrap().join("collection.media");
        let media_dir = temp_dir.path().join("collection.media");

        if fixture_media.exists() {
            copy_dir_all(&fixture_media, &media_dir)
                .context("Failed to copy media directory")?;
        } else {
            std::fs::create_dir_all(&media_dir)
                .context("Failed to create media directory")?;
        }

        Ok(Self {
            _temp_dir: temp_dir,
            collection_path,
            media_dir,
        })
    }

    /// Get path to the fixture collection
    fn fixture_collection_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/test_collection/collection.anki2")
    }

    /// Open repository for this test collection
    pub fn open_repository(&self) -> Result<AnkiRepository> {
        AnkiRepository::new(&self.collection_path)
    }
}

/// Recursively copy directory contents
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(entry.path(), &dst_path)?;
        }
    }
    Ok(())
}

/// Known test note IDs from golden dataset
#[allow(dead_code)]
pub mod test_notes {
    // Notes with images - good for testing media path resolution
    pub const DAG_NOTE: i64 = 1695797540370;  // Has dag.png image
    pub const STAR_SCHEMA: i64 = 1713763428669;  // Has star-schema.png image
    pub const MERCATOR: i64 = 1737647330399;  // Has mercator.png and wsg-enu2.png images

    // Text-heavy notes - good for testing content rendering
    pub const TREE: i64 = 1695797540371;
    pub const RECURSIVE_DFS: i64 = 1695797540372;
    pub const TAIL_RECURSION: i64 = 1698125272387;
    pub const BIG_O: i64 = 1713934919822;

    // Data science notes - good for testing HTML formatting
    pub const F1_SCORE: i64 = 1714489634039;
    pub const ACCURACY: i64 = 1714489634040;
    pub const COLBERT: i64 = 1715928977633;

    // Additional notes
    pub const SCHEMA_REASONING: i64 = 1726838512787;
    pub const RRF: i64 = 1727071084388;
    pub const AGENT: i64 = 1748163225945;
    pub const IMBALANCED: i64 = 1748169001421;

    // For testing error cases
    pub const NONEXISTENT: i64 = 999999999;
}
