use ankiview::cli::args::{Args, Command};
use clap::Parser;

#[test]
fn given_note_id_only_when_parsing_then_defaults_to_view() {
    // Arrange
    let args = vec!["ankiview", "1234567890"];

    // Act
    let parsed = Args::try_parse_from(args).unwrap();
    let command = parsed.resolve_command();

    // Assert
    match command {
        Command::View {
            note_id,
            collection,
            profile,
        } => {
            assert_eq!(note_id, 1234567890);
            assert_eq!(collection, None);
            assert_eq!(profile, None);
        }
        _ => panic!("Expected View command"),
    }
}

#[test]
fn given_explicit_view_command_when_parsing_then_succeeds() {
    // Arrange
    let args = vec!["ankiview", "view", "1234567890"];

    // Act
    let parsed = Args::try_parse_from(args).unwrap();

    // Assert
    match parsed.command.unwrap() {
        Command::View {
            note_id,
            collection,
            profile,
        } => {
            assert_eq!(note_id, 1234567890);
            assert_eq!(collection, None);
            assert_eq!(profile, None);
        }
        _ => panic!("Expected View command"),
    }
}

#[test]
fn given_delete_command_when_parsing_then_succeeds() {
    // Arrange
    let args = vec!["ankiview", "delete", "1234567890"];

    // Act
    let parsed = Args::try_parse_from(args).unwrap();

    // Assert
    match parsed.command.unwrap() {
        Command::Delete {
            note_id,
            collection,
            profile,
        } => {
            assert_eq!(note_id, 1234567890);
            assert_eq!(collection, None);
            assert_eq!(profile, None);
        }
        _ => panic!("Expected Delete command"),
    }
}

#[test]
fn given_delete_with_collection_flag_when_parsing_then_succeeds() {
    // Arrange
    let args = vec![
        "ankiview",
        "delete",
        "-c",
        "/path/to/collection.anki2",
        "1234567890",
    ];

    // Act
    let parsed = Args::try_parse_from(args).unwrap();

    // Assert
    match parsed.command.unwrap() {
        Command::Delete {
            note_id,
            collection,
            profile,
        } => {
            assert_eq!(note_id, 1234567890);
            assert_eq!(
                collection,
                Some(std::path::PathBuf::from("/path/to/collection.anki2"))
            );
            assert_eq!(profile, None);
        }
        _ => panic!("Expected Delete command"),
    }
}

#[test]
fn given_view_with_profile_flag_when_parsing_then_succeeds() {
    // Arrange
    let args = vec!["ankiview", "view", "-p", "User 1", "1234567890"];

    // Act
    let parsed = Args::try_parse_from(args).unwrap();

    // Assert
    match parsed.command.unwrap() {
        Command::View {
            note_id,
            collection,
            profile,
        } => {
            assert_eq!(note_id, 1234567890);
            assert_eq!(collection, None);
            assert_eq!(profile, Some("User 1".to_string()));
        }
        _ => panic!("Expected View command"),
    }
}

#[test]
fn given_verbose_flag_when_parsing_then_increments_count() {
    // Arrange
    let args = vec!["ankiview", "-vv", "1234567890"];

    // Act
    let parsed = Args::try_parse_from(args).unwrap();

    // Assert
    assert_eq!(parsed.verbose, 2);
}
