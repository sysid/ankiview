#!/usr/bin/env python3
"""
Clean unused media files from Anki collection.

This script:
1. Scans all notes in collection.anki2 to find media references
2. Compares with actual files in collection.media/
3. Deletes unreferenced files
4. Updates collection.media.db2 to match
"""

import sqlite3
import re
import os
import sys
import argparse
from pathlib import Path
from typing import Set

def extract_media_from_html(html: str) -> Set[str]:
    """Extract all media filenames referenced in HTML content."""
    media_files = set()

    # Pattern 1: <img src="filename">
    img_pattern = r'<img[^>]+src=["\']([^"\']+)["\']'
    for match in re.finditer(img_pattern, html, re.IGNORECASE):
        filename = match.group(1)
        # Skip external URLs
        if not filename.startswith(('http://', 'https://', '//', 'data:')):
            media_files.add(filename)

    # Pattern 2: [sound:filename]
    sound_pattern = r'\[sound:([^\]]+)\]'
    for match in re.finditer(sound_pattern, html, re.IGNORECASE):
        media_files.add(match.group(1))

    # Pattern 3: Background images in style attributes
    bg_pattern = r'background-image:\s*url\(["\']?([^"\')\s]+)["\']?\)'
    for match in re.finditer(bg_pattern, html, re.IGNORECASE):
        filename = match.group(1)
        if not filename.startswith(('http://', 'https://', '//', 'data:')):
            media_files.add(filename)

    return media_files

def get_referenced_media(collection_path: Path) -> Set[str]:
    """Get all media files referenced in notes."""
    print(f"Analyzing notes in {collection_path}...")
    conn = sqlite3.connect(collection_path)
    cursor = conn.cursor()

    # Get all note fields
    cursor.execute("SELECT flds FROM notes")

    referenced = set()
    note_count = 0
    for (flds,) in cursor:
        note_count += 1
        # Fields are separated by \x1f
        media = extract_media_from_html(flds)
        referenced.update(media)

    conn.close()
    print(f"  Found {note_count} notes")
    print(f"  Found {len(referenced)} unique media references")

    return referenced

def get_actual_media_files(media_dir: Path) -> Set[str]:
    """Get all actual files in the media directory."""
    print(f"\nScanning media directory: {media_dir}...")
    files = set()

    for item in media_dir.iterdir():
        if item.is_file():
            files.add(item.name)

    print(f"  Found {len(files)} files")
    return files

def clean_media_database(db_path: Path, referenced_files: Set[str]):
    """Update media database to remove unreferenced entries."""
    print(f"\nUpdating media database: {db_path}...")
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()

    # Get current entries
    cursor.execute("SELECT fname FROM media WHERE csum IS NOT NULL")
    db_entries = {row[0] for row in cursor}

    # Find entries to remove
    to_remove = db_entries - referenced_files

    if to_remove:
        print(f"  Removing {len(to_remove)} entries from media database")
        for fname in to_remove:
            cursor.execute("DELETE FROM media WHERE fname = ?", (fname,))
        conn.commit()
    else:
        print("  No database entries to remove")

    conn.close()

def main():
    # Parse arguments
    parser = argparse.ArgumentParser(
        description="Clean unused media files from Anki collection"
    )
    parser.add_argument(
        "--yes", "-y",
        action="store_true",
        help="Skip confirmation prompt and proceed with deletion"
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would be deleted without actually deleting"
    )
    args = parser.parse_args()

    # Paths
    base_dir = Path("/Users/Q187392/dev/s/private/ankiview/data/testuser")
    collection_path = base_dir / "collection.anki2"
    media_dir = base_dir / "collection.media"
    media_db_path = base_dir / "collection.media.db2"

    print("=" * 70)
    print("Anki Media Cleanup")
    print("=" * 70)

    # Step 1: Find referenced media
    referenced = get_referenced_media(collection_path)

    # Step 2: Find actual files
    actual_files = get_actual_media_files(media_dir)

    # Step 3: Identify unreferenced files
    unreferenced = actual_files - referenced

    print("\n" + "=" * 70)
    print(f"Summary:")
    print(f"  Referenced media files: {len(referenced)}")
    print(f"  Actual media files:     {len(actual_files)}")
    print(f"  Unreferenced files:     {len(unreferenced)}")
    print("=" * 70)

    if not unreferenced:
        print("\n✓ No unreferenced files to delete!")
        return

    # Show first 20 files to be deleted
    print("\nFiles to be deleted:")
    for i, filename in enumerate(sorted(unreferenced), 1):
        if i <= 20:
            print(f"  {filename}")
        elif i == 21:
            print(f"  ... and {len(unreferenced) - 20} more")
            break

    # Dry run mode - exit early
    if args.dry_run:
        print("\n✓ Dry run mode - no files deleted")
        return

    # Confirm deletion
    if not args.yes:
        try:
            response = input("\nProceed with deletion? (yes/no): ")
            if response.lower() != 'yes':
                print("Aborted.")
                return
        except (EOFError, KeyboardInterrupt):
            print("\nAborted.")
            return
    else:
        print("\nAuto-confirming deletion (--yes flag used)")

    # Step 4: Delete unreferenced files
    print("\nDeleting unreferenced files...")
    deleted_count = 0
    for filename in unreferenced:
        file_path = media_dir / filename
        try:
            file_path.unlink()
            deleted_count += 1
        except Exception as e:
            print(f"  Error deleting {filename}: {e}")

    print(f"  Deleted {deleted_count} files")

    # Step 5: Update media database
    clean_media_database(media_db_path, referenced)

    print("\n✓ Cleanup completed successfully!")
    print(f"  Disk space freed: ~{deleted_count} files")

if __name__ == "__main__":
    main()
