# AnkiView 🎴

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

AnkiView is a command-line tool that lets you quickly view Anki notes directly from your collection file, without needing to open the Anki application. Perfect for quick information gathering.

## Features ✨

- View any note by its ID in your default browser
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

View a note by its ID:

```bash
ankiview 1234567890
```

Use a specific collection file:

```bash
ankiview -c /path/to/collection.anki2 1234567890
```

Specify an Anki profile:

```bash
ankiview -p "User 1" 1234567890
```

Enable debug logging:

```bash
ankiview -v 1234567890
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
   - Make sure Anki isn't running
   - Check file permissions

## Contributing 🤝

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License 📄

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments 🙏

- [Anki](https://apps.ankiweb.net/) - The amazing flashcard program
- [anki-core](https://crates.io/crates/anki) - Rust bindings for Anki
