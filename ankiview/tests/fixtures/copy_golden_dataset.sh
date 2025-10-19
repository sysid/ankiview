#!/usr/bin/env bash
# Copy golden test dataset to fixtures directory
# This script should be run from the repository root

set -euo pipefail

GOLDEN_SOURCE="/Users/Q187392/dev/s/private/ankiview/data/testuser"
FIXTURE_TARGET="ankiview/tests/fixtures/test_collection"

echo "Copying golden dataset to test fixtures..."

# Remove old fixture if exists
if [ -d "$FIXTURE_TARGET" ]; then
    echo "Removing existing fixture at $FIXTURE_TARGET"
    rm -rf "$FIXTURE_TARGET"
fi

# Create fixture directory
mkdir -p "$FIXTURE_TARGET"

# Copy collection file (close any open SQLite connections first)
echo "Copying collection.anki2..."
cp "$GOLDEN_SOURCE/collection.anki2" "$FIXTURE_TARGET/"

# Copy media directory
echo "Copying media files..."
cp -r "$GOLDEN_SOURCE/collection.media" "$FIXTURE_TARGET/"

# Copy media database
echo "Copying media database..."
cp "$GOLDEN_SOURCE/collection.media.db2" "$FIXTURE_TARGET/"

# Verify files were copied
echo ""
echo "Verification:"
ls -lh "$FIXTURE_TARGET/collection.anki2"
ls -lh "$FIXTURE_TARGET/collection.media.db2"
echo ""
echo "Media files:"
ls -lh "$FIXTURE_TARGET/collection.media/"
echo ""
echo "Golden dataset copied successfully!"
echo ""
echo "IMPORTANT: Do not modify files in $GOLDEN_SOURCE"
echo "Tests will work with copies of this fixture."
