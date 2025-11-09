use ankiview::cli::args::{Args, Command};
use clap::Parser;

#[test]
fn given_no_subcommand_when_parsing_then_fails() {
    // Arrange
    let args = vec!["ankiview", "1234567890"];

    // Act & Assert
    let result = Args::try_parse_from(args);
    assert!(result.is_err(), "Should fail without subcommand");
}

#[test]
fn given_explicit_view_command_when_parsing_then_succeeds() {
    // Arrange
    let args = vec!["ankiview", "view", "1234567890"];

    // Act
    let parsed = Args::try_parse_from(args).unwrap();

    // Assert
    match parsed.command {
        Command::View { note_id, json } => {
            assert_eq!(note_id, 1234567890);
            assert!(!json);
        }
        _ => panic!("Expected View command"),
    }
    assert_eq!(parsed.collection, None);
    assert_eq!(parsed.profile, None);
}

#[test]
fn given_delete_command_when_parsing_then_succeeds() {
    // Arrange
    let args = vec!["ankiview", "delete", "1234567890"];

    // Act
    let parsed = Args::try_parse_from(args).unwrap();

    // Assert
    match parsed.command {
        Command::Delete { note_id } => {
            assert_eq!(note_id, 1234567890);
        }
        _ => panic!("Expected Delete command"),
    }
    assert_eq!(parsed.collection, None);
    assert_eq!(parsed.profile, None);
}

#[test]
fn given_global_collection_flag_when_parsing_then_succeeds() {
    // Arrange
    let args = vec![
        "ankiview",
        "-c",
        "/path/to/collection.anki2",
        "delete",
        "1234567890",
    ];

    // Act
    let parsed = Args::try_parse_from(args).unwrap();

    // Assert
    match parsed.command {
        Command::Delete { note_id } => {
            assert_eq!(note_id, 1234567890);
        }
        _ => panic!("Expected Delete command"),
    }
    assert_eq!(
        parsed.collection,
        Some(std::path::PathBuf::from("/path/to/collection.anki2"))
    );
    assert_eq!(parsed.profile, None);
}

#[test]
fn given_global_profile_flag_when_parsing_then_succeeds() {
    // Arrange
    let args = vec!["ankiview", "-p", "User 1", "view", "1234567890"];

    // Act
    let parsed = Args::try_parse_from(args).unwrap();

    // Assert
    match parsed.command {
        Command::View { note_id, json } => {
            assert_eq!(note_id, 1234567890);
            assert!(!json);
        }
        _ => panic!("Expected View command"),
    }
    assert_eq!(parsed.collection, None);
    assert_eq!(parsed.profile, Some("User 1".to_string()));
}

#[test]
fn given_verbose_flag_when_parsing_then_increments_count() {
    // Arrange
    let args = vec!["ankiview", "-vv", "view", "1234567890"];

    // Act
    let parsed = Args::try_parse_from(args).unwrap();

    // Assert
    assert_eq!(parsed.verbose, 2);
}

#[test]
fn given_collection_flag_after_subcommand_when_parsing_then_succeeds() {
    // Arrange - global flags work anywhere when marked as global
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
    match parsed.command {
        Command::Delete { note_id } => {
            assert_eq!(note_id, 1234567890);
        }
        _ => panic!("Expected Delete command"),
    }
    assert_eq!(
        parsed.collection,
        Some(std::path::PathBuf::from("/path/to/collection.anki2"))
    );
}

#[test]
fn given_json_flag_when_parsing_view_command_then_json_is_true() {
    // Arrange
    let args = vec!["ankiview", "view", "--json", "1234567890"];

    // Act
    let parsed = Args::try_parse_from(args).unwrap();

    // Assert
    match parsed.command {
        Command::View { note_id, json } => {
            assert_eq!(note_id, 1234567890);
            assert!(json);
        }
        _ => panic!("Expected View command"),
    }
}

#[test]
fn given_no_json_flag_when_parsing_view_command_then_json_is_false() {
    // Arrange
    let args = vec!["ankiview", "view", "1234567890"];

    // Act
    let parsed = Args::try_parse_from(args).unwrap();

    // Assert
    match parsed.command {
        Command::View { note_id, json } => {
            assert_eq!(note_id, 1234567890);
            assert!(!json);
        }
        _ => panic!("Expected View command"),
    }
}

#[test]
fn given_json_flag_with_global_flags_when_parsing_then_succeeds() {
    // Arrange
    let args = vec!["ankiview", "-v", "view", "--json", "1234567890"];

    // Act
    let parsed = Args::try_parse_from(args).unwrap();

    // Assert
    match parsed.command {
        Command::View { note_id, json } => {
            assert_eq!(note_id, 1234567890);
            assert!(json);
        }
        _ => panic!("Expected View command"),
    }
    assert_eq!(parsed.verbose, 1);
}

#[test]
fn given_list_command_without_search_when_parsing_then_succeeds() {
    // Arrange
    let args = vec!["ankiview", "list"];

    // Act
    let parsed = Args::try_parse_from(args).unwrap();

    // Assert
    match parsed.command {
        Command::List { search } => {
            assert_eq!(search, None);
        }
        _ => panic!("Expected List command"),
    }
}

#[test]
fn given_list_command_with_search_when_parsing_then_succeeds() {
    // Arrange
    let args = vec!["ankiview", "list", "tree"];

    // Act
    let parsed = Args::try_parse_from(args).unwrap();

    // Assert
    match parsed.command {
        Command::List { search } => {
            assert_eq!(search, Some("tree".to_string()));
        }
        _ => panic!("Expected List command"),
    }
}

#[test]
fn given_list_command_with_global_flags_when_parsing_then_succeeds() {
    // Arrange
    let args = vec!["ankiview", "-v", "list", "graph"];

    // Act
    let parsed = Args::try_parse_from(args).unwrap();

    // Assert
    match parsed.command {
        Command::List { search } => {
            assert_eq!(search, Some("graph".to_string()));
        }
        _ => panic!("Expected List command"),
    }
    assert_eq!(parsed.verbose, 1);
}

#[test]
fn given_list_card_types_command_when_parsing_then_succeeds() {
    // Arrange
    let args = vec!["ankiview", "list-card-types"];

    // Act
    let parsed = Args::try_parse_from(args).unwrap();

    // Assert
    match parsed.command {
        Command::ListCardTypes => {}
        _ => panic!("Expected ListCardTypes command"),
    }
}

#[test]
fn given_collect_with_card_type_when_parsing_then_succeeds() {
    // Arrange
    let args = vec![
        "ankiview",
        "collect",
        "--card-type",
        "Inka Basic",
        "notes.md",
    ];

    // Act
    let parsed = Args::try_parse_from(args).unwrap();

    // Assert
    match parsed.command {
        Command::Collect { path, card_type, .. } => {
            assert_eq!(path, std::path::PathBuf::from("notes.md"));
            assert_eq!(card_type, Some("Inka Basic".to_string()));
        }
        _ => panic!("Expected Collect command"),
    }
}

#[test]
fn given_collect_without_card_type_when_parsing_then_defaults_to_none() {
    // Arrange
    let args = vec!["ankiview", "collect", "notes.md"];

    // Act
    let parsed = Args::try_parse_from(args).unwrap();

    // Assert
    match parsed.command {
        Command::Collect { path, card_type, .. } => {
            assert_eq!(path, std::path::PathBuf::from("notes.md"));
            assert_eq!(card_type, None);
        }
        _ => panic!("Expected Collect command"),
    }
}
