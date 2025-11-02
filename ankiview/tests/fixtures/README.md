# Test Fixtures

## Golden Test Dataset

**Source**: `/Users/Q187392/dev/s/private/ankiview/data/testuser/`
**Fixture Location**: `test_collection/User 1/`

**IMPORTANT**: The golden dataset in the source location is READ-ONLY. Never modify it. All tests work with copies.

### Structure
- 15 notes with real-world content
- Basic card type (front/back)
- 4 media files (PNG images)
- Collection size: ~1MB
- Media directory: ~140KB

### Content Coverage
- Data structures (DAG, Tree, DFS)
- Algorithms and complexity
- Data science metrics (F1, accuracy)
- Database concepts (star schema)
- Embeddings and ML concepts
- Geographic reference systems

### Media Files
- `dag.png` (37KB) - Referenced by note 1695797540370
- `star-schema.png` (16KB) - Referenced by note 1713763428669
- `mercator.png` (24KB) - Referenced by note 1737647330399
- `wsg-enu2.png` (58KB) - Referenced by note 1737647330399

### Refreshing Fixture from Golden Dataset

If the golden dataset is updated, refresh the fixture:

```bash
chmod +x ankiview/tests/fixtures/copy_golden_dataset.sh
./ankiview/tests/fixtures/copy_golden_dataset.sh
```

### Note IDs for Testing

Use these note IDs in integration tests:

```rust
pub mod test_notes {
    // Notes with images
    pub const DAG_NOTE: i64 = 1695797540370;
    pub const STAR_SCHEMA: i64 = 1713763428669;
    pub const MERCATOR: i64 = 1737647330399;

    // Text-heavy notes
    pub const TREE: i64 = 1695797540371;
    pub const RECURSIVE_DFS: i64 = 1695797540372;
    pub const TAIL_RECURSION: i64 = 1698125272387;

    // Data science notes
    pub const F1_SCORE: i64 = 1714489634039;
    pub const ACCURACY: i64 = 1714489634040;
    pub const COLBERT: i64 = 1715928977633;

    // For testing errors
    pub const NONEXISTENT: i64 = 999999999;
}
```
