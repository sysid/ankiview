use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};
use std::path::Path;

/// TOML configuration for inka collection
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct Config {
    #[serde(default)]
    pub defaults: Defaults,
    #[serde(default)]
    pub anki: AnkiConfig,
    #[serde(default)]
    pub highlight: HighlightConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Defaults {
    #[serde(default = "default_profile")]
    pub profile: String,
    #[serde(default = "default_deck")]
    pub deck: String,
    #[serde(default = "default_folder")]
    pub folder: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AnkiConfig {
    #[serde(default = "default_path")]
    pub path: String,
    #[serde(default = "default_basic_type")]
    pub basic_type: String,
    #[serde(default = "default_front_field")]
    pub front_field: String,
    #[serde(default = "default_back_field")]
    pub back_field: String,
    #[serde(default = "default_cloze_type")]
    pub cloze_type: String,
    #[serde(default = "default_cloze_field")]
    pub cloze_field: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct HighlightConfig {
    #[serde(default = "default_highlight_style")]
    pub style: String,
}

// Default value functions
fn default_profile() -> String { String::new() }
fn default_deck() -> String { "Default".to_string() }
fn default_folder() -> String { String::new() }
fn default_path() -> String { String::new() }
fn default_basic_type() -> String { "Inka Basic".to_string() }
fn default_front_field() -> String { "Front".to_string() }
fn default_back_field() -> String { "Back".to_string() }
fn default_cloze_type() -> String { "Inka Cloze".to_string() }
fn default_cloze_field() -> String { "Text".to_string() }
fn default_highlight_style() -> String { "monokai".to_string() }

impl Default for Defaults {
    fn default() -> Self {
        Self {
            profile: default_profile(),
            deck: default_deck(),
            folder: default_folder(),
        }
    }
}

impl Default for AnkiConfig {
    fn default() -> Self {
        Self {
            path: default_path(),
            basic_type: default_basic_type(),
            front_field: default_front_field(),
            back_field: default_back_field(),
            cloze_type: default_cloze_type(),
            cloze_field: default_cloze_field(),
        }
    }
}

impl Default for HighlightConfig {
    fn default() -> Self {
        Self {
            style: default_highlight_style(),
        }
    }
}

impl Config {
    /// Load configuration from TOML file
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .context("Failed to read config file")?;

        let config: Config = toml::from_str(&content)
            .context("Failed to parse TOML config")?;

        Ok(config)
    }

    /// Save configuration to TOML file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let toml_string = toml::to_string_pretty(self)
            .context("Failed to serialize config to TOML")?;

        std::fs::write(path.as_ref(), toml_string)
            .context("Failed to write config file")?;

        Ok(())
    }

    /// Create default configuration file at path
    pub fn create_default(path: impl AsRef<Path>) -> Result<Self> {
        let config = Self::default();
        config.save(path)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn given_no_file_when_creating_default_then_creates_with_defaults() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("inka.toml");

        let config = Config::create_default(&config_path).unwrap();

        assert_eq!(config.defaults.deck, "Default");
        assert_eq!(config.anki.basic_type, "Inka Basic");
        assert_eq!(config.highlight.style, "monokai");
        assert!(config_path.exists());
    }

    #[test]
    fn given_config_when_saving_then_writes_toml_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.toml");

        let config = Config::default();
        config.save(&config_path).unwrap();

        assert!(config_path.exists());
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("[defaults]"));
        assert!(content.contains("[anki]"));
        assert!(content.contains("[highlight]"));
    }

    #[test]
    fn given_toml_file_when_loading_then_reads_values() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("load_test.toml");

        let toml_content = r#"
[defaults]
profile = "User 1"
deck = "TestDeck"
folder = "/path/to/notes"

[anki]
path = "/custom/collection.anki2"
basic_type = "Custom Basic"
front_field = "Question"
back_field = "Answer"
cloze_type = "Custom Cloze"
cloze_field = "Content"

[highlight]
style = "github"
"#;
        fs::write(&config_path, toml_content).unwrap();

        let config = Config::load(&config_path).unwrap();

        assert_eq!(config.defaults.profile, "User 1");
        assert_eq!(config.defaults.deck, "TestDeck");
        assert_eq!(config.defaults.folder, "/path/to/notes");
        assert_eq!(config.anki.path, "/custom/collection.anki2");
        assert_eq!(config.anki.basic_type, "Custom Basic");
        assert_eq!(config.highlight.style, "github");
    }

    #[test]
    fn given_partial_toml_when_loading_then_uses_defaults() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("partial.toml");

        let toml_content = r#"
[defaults]
deck = "MyDeck"
"#;
        fs::write(&config_path, toml_content).unwrap();

        let config = Config::load(&config_path).unwrap();

        // Specified value
        assert_eq!(config.defaults.deck, "MyDeck");
        // Default values
        assert_eq!(config.defaults.profile, "");
        assert_eq!(config.anki.basic_type, "Inka Basic");
        assert_eq!(config.highlight.style, "monokai");
    }

    #[test]
    fn given_nonexistent_file_when_loading_then_returns_error() {
        let result = Config::load("/nonexistent/path/config.toml");

        assert!(result.is_err());
    }

    #[test]
    fn given_round_trip_when_saving_and_loading_then_preserves_values() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("roundtrip.toml");

        let original = Config {
            defaults: Defaults {
                profile: "Test Profile".to_string(),
                deck: "Test Deck".to_string(),
                folder: "/test/folder".to_string(),
            },
            anki: AnkiConfig {
                path: "/test/collection.anki2".to_string(),
                ..Default::default()
            },
            highlight: HighlightConfig {
                style: "nord".to_string(),
            },
        };

        original.save(&config_path).unwrap();
        let loaded = Config::load(&config_path).unwrap();

        assert_eq!(loaded, original);
    }
}
