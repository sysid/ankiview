# AnkiView 🎴

AnkiView is a command-line tool that lets you quickly view Anki notes directly from your collection
file, without needing to open the Anki application. Perfect for quick information gathering.

> It serves also as a much more powerfull drop-in replacement for [inka2](https://github.com/sysid/inka2?tab=readme-ov-file).

## Features ✨

- **View notes** - View any note by its ID in your default browser
- **Delete notes** - Delete notes from your collection via CLI
- **List notes** - Browse and search notes from the command line
- **List card types** - See available card types in your collection
- **Import markdown** - Convert markdown flashcards to Anki notes
- **Smart updates** - Automatically track cards with ID comments
- **Media handling** - Import images from markdown files
- **Hash caching** - Skip unchanged files for fast re-imports
- **Custom card types** - Use any card type from your collection
- Automatic collection file detection
- Support for multiple Anki profiles
- LaTeX math rendering support
- Clean, modern card presentation
- Cross-platform support (Windows, macOS, Linux)

## Installation 🚀

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/ankiview
cd ankiview

# Build and install
cargo install --path .
```

### Prerequisites

- Rust 1.70 or higher
- An Anki installation with at least one profile

## Usage 💡

### View a note

View a note by its ID:

```bash
ankiview view 1234567890
```

Use a specific collection file:

```bash
ankiview -c /path/to/collection.anki2 view 1234567890
```

Specify an Anki profile:

```bash
ankiview -p "User 1" view 1234567890
```

### Delete a note

Delete a note by its ID:

```bash
ankiview delete 1234567890
```

**Warning:** Deletion is permanent and will remove the note and all associated cards from your collection.

Global flags work with all commands:

```bash
ankiview -c /path/to/collection.anki2 delete 1234567890
ankiview -p "User 1" delete 1234567890
```

### List notes

List all notes in your collection with their IDs and first line of content:

```bash
ankiview list
```

Filter notes by searching the front field:

```bash
ankiview list "rust programming"
```

This is useful for:
- Finding note IDs when you know the content
- Browsing your collection from the command line
- Quick searches without opening Anki

### List available card types

List all card types (notetypes) available in your Anki collection:

```bash
ankiview list-card-types
```

This shows you which card types you can use with the `--card-type` flag.

### Collect markdown cards

Import markdown flashcards into your Anki collection:

```bash
# Import a single file
ankiview collect notes.md

# Import a directory (non-recursive)
ankiview collect notes/

# Import recursively (all subdirectories)
ankiview collect -r notes/

# Use a specific card type
ankiview collect --card-type "Basic" notes.md
```

**Markdown Format**

Basic cards (question and answer):
```markdown
---
Deck: Programming
Tags: rust basics

1. What is Rust?
> A systems programming language

2. What is Cargo?
> Rust's package manager
---
```

Cloze deletion cards:
```markdown
---
Deck: Programming

1. Rust provides {memory safety} without garbage collection.
2. The {{c1::borrow checker}} ensures {{c2::safe concurrency}}.
---
```

Cards with images:
```markdown
---
Deck: ComputerScience

1. What type of graph is this?
> ![Graph diagram](images/dag.png)
> A directed acyclic graph (DAG)
---
```

**How It Works**

1. AnkiView reads your markdown files
2. Creates or updates notes in Anki
3. Injects ID comments into your markdown for tracking
4. Copies media files to Anki's collection.media/

After the first run, your markdown will have ID comments:
```markdown
<!--ID:1686433857327-->
1. What is Rust?
> A systems programming language
```

This allows you to edit the content and re-run collect to update (not duplicate) the cards.

**Advanced Usage**

```bash
# Use a specific card type (defaults to "Inka Basic")
ankiview collect --card-type "Basic" notes/

# Recover lost IDs by searching Anki
ankiview collect -u notes/

# Force rebuild (bypass cache)
ankiview collect -f notes/

# Overwrite existing media files
ankiview collect --force notes/

# Continue on errors, report at end
ankiview collect -i notes/

# Combine flags for batch processing
ankiview collect -ri --card-type "Basic" notes/
```

**Flag Reference**

| Flag | Description |
|------|-------------|
| `-r, --recursive` | Process subdirectories |
| `--force` | Overwrite conflicting media files |
| `-i, --ignore-errors` | Continue processing on errors |
| `-f, --full-sync` | Bypass hash cache (force rebuild) |
| `-u, --update-ids` | Search Anki for existing notes by content |
| `--card-type TYPE` | Use specific card type (defaults to "Inka Basic") |

**Performance Note:** AnkiView maintains a hash cache to skip unchanged files. Use `-f` to force processing all files.

### Debug logging

Enable debug logging for any command (global flags can appear before or after subcommand):

```bash
ankiview -v delete 1234567890      # DEBUG level
ankiview -vv view 1234567890       # TRACE level
ankiview delete -v 1234567890      # Also works
```

## How It Works 🔧

AnkiView:
1. Locates your Anki collection file
2. Opens the collection safely (read-only)
3. Retrieves the specified note
4. Generates a beautiful HTML preview
5. Opens it in your default browser

## Example Output 📝

When you view a note, you'll see:
- The question and answer clearly separated
- Properly rendered LaTeX equations
- Card tags and metadata
- Clean, modern styling

## Development 🛠

The project structure:

```
src/
├── application/     # Use cases and business logic
├── cli/            # Command-line interface
├── domain/         # Core domain models
├── infrastructure/ # External interfaces (Anki, browser)
└── ports/          # Input/output adapters
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with logging
RUST_LOG=debug cargo test
```

## Troubleshooting 🔍

### Common Issues

1. **"Collection file not found"**
   - Ensure Anki is installed
   - Check if the profile name is correct
   - Verify the collection path

2. **"Failed to open Anki collection"**
   - Make sure Anki isn't running (required for all commands)
   - Check file permissions

3. **"Notetype '...' not found"** (collect command)
   - The specified card type doesn't exist in your collection
   - Run `ankiview list-card-types` to see available types
   - Omit `--card-type` to use the default "Inka Basic"
   - Create the card type in Anki first if needed

4. **"Different file with the same name already exists"** (collect command)
   - Media file conflict detected
   - Use `--force` flag to overwrite existing media files
   - Or rename your image file to avoid conflict

5. **Duplicate cards created** (collect command)
   - Ensure ID comments (`<!--ID:-->`) are preserved in markdown
   - Use `--update-ids` flag to recover lost IDs
   - Check that you didn't manually modify or remove ID comments

6. **Cards not updating** (collect command)
   - File may be unchanged (check hash cache)
   - Use `-f` flag to force rebuild
   - Verify ID comments are correct and match Anki notes

## Contributing 🤝

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## Acknowledgments 🙏

- [Anki](https://apps.ankiweb.net/) - The amazing flashcard program
