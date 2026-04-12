// src/infrastructure/note_template.rs
//
// Editor template for note editing, following bkmr's section-delimited pattern.
// Renders a note to a structured template, parses it back, validates against Anki invariants.

use crate::domain::Note;
use anyhow::{bail, Result};
use regex::Regex;

/// Represents a note opened for editing in the user's $EDITOR.
#[derive(Debug, Clone)]
pub struct NoteTemplate {
    pub note_id: i64,
    pub note_type_name: String,
    pub field_names: Vec<String>,
    pub field_values: Vec<String>,
    pub tags: Vec<String>,
}

impl NoteTemplate {
    /// Build a template from a domain Note.
    /// Field names are inferred from the model name.
    pub fn from_note(note: &Note) -> Self {
        let (field_names, field_values) = infer_fields(note);
        Self {
            note_id: note.id,
            note_type_name: note.model_name.clone(),
            field_names,
            field_values,
            tags: note.tags.clone(),
        }
    }

    /// Render the template to a string for writing to a temp file.
    pub fn to_string(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "# Note {} ({})\n",
            self.note_id, self.note_type_name
        ));
        out.push_str("# Lines starting with '#' are comments and will be ignored.\n");
        out.push_str("# Section markers (=== NAME ===) must not be removed.\n\n");

        out.push_str("=== ID ===\n");
        out.push_str(&format!("{}\n\n", self.note_id));

        out.push_str("=== NOTE TYPE ===\n");
        out.push_str(&format!("{}\n\n", self.note_type_name));

        for (name, value) in self.field_names.iter().zip(self.field_values.iter()) {
            out.push_str(&format!("=== {} ===\n", name.to_uppercase()));
            out.push_str(value);
            if !value.ends_with('\n') {
                out.push('\n');
            }
            out.push('\n');
        }

        out.push_str("=== TAGS ===\n");
        out.push_str(&self.tags.join(" "));
        out.push_str("\n\n");

        out.push_str("=== END ===\n");
        out
    }

    /// Parse an edited template string back into a NoteTemplate.
    /// Validates structure and read-only fields against the original note.
    pub fn from_string(text: &str, original: &Note) -> Result<Self> {
        let sections = parse_sections(text)?;

        // Validate ID matches
        let id_str = sections
            .get("ID")
            .ok_or_else(|| anyhow::anyhow!("Missing section marker: === ID ==="))?
            .trim();
        let parsed_id: i64 = id_str
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid note ID in template: '{}'", id_str))?;
        if parsed_id != original.id {
            bail!(
                "Note ID mismatch: template says {}, expected {}",
                parsed_id,
                original.id
            );
        }

        // Validate note type matches
        let note_type = sections
            .get("NOTE TYPE")
            .ok_or_else(|| anyhow::anyhow!("Missing section marker: === NOTE TYPE ==="))?
            .trim();
        if note_type != original.model_name {
            bail!("Note type cannot be changed via edit");
        }

        // Extract fields based on original note's field structure
        let (field_names, _) = infer_fields(original);
        let mut field_values = Vec::new();
        for name in &field_names {
            let value = sections
                .get(name.to_uppercase().as_str())
                .ok_or_else(|| {
                    anyhow::anyhow!("Missing section marker: === {} ===", name.to_uppercase())
                })?;
            // Trim trailing newlines but preserve internal content
            field_values.push(value.trim_end_matches('\n').to_string());
        }

        // Extract tags
        let tags_str = sections.get("TAGS").map(|s| s.trim()).unwrap_or("");
        let tags: Vec<String> = if tags_str.is_empty() {
            vec![]
        } else {
            tags_str.split_whitespace().map(|s| s.to_string()).collect()
        };

        Ok(Self {
            note_id: original.id,
            note_type_name: original.model_name.clone(),
            field_names,
            field_values,
            tags,
        })
    }

    /// Validate the template against Anki invariants.
    pub fn validate(&self, original: &Note) -> Result<()> {
        // First field (sort field) cannot be empty
        if let Some(first_value) = self.field_values.first() {
            if first_value.trim().is_empty() {
                let field_name = self.field_names.first().map(|s| s.as_str()).unwrap_or("First");
                bail!(
                    "Field '{}' cannot be empty — it is the sort field for this note type",
                    field_name
                );
            }
        }

        // Cloze notes must have cloze deletions in the Text field
        let is_cloze = original.model_name.to_lowercase().contains("cloze");
        if is_cloze {
            if let Some(text_value) = self.field_values.first() {
                let cloze_re = Regex::new(r"\{\{c\d+::").unwrap();
                if !cloze_re.is_match(text_value) {
                    bail!(
                        "Cloze note Text field must contain at least one cloze deletion (e.g., {{{{c1::answer}}}})"
                    );
                }
            }
        }

        Ok(())
    }

    /// Convert to fields and tags for the repository update call.
    pub fn to_update(&self) -> (Vec<String>, Vec<String>) {
        (self.field_values.clone(), self.tags.clone())
    }
}

/// Infer field names and values from a Note based on its model name.
fn infer_fields(note: &Note) -> (Vec<String>, Vec<String>) {
    let model_lower = note.model_name.to_lowercase();
    if model_lower.contains("cloze") {
        (
            vec!["Text".to_string(), "Extra".to_string()],
            vec![note.front.clone(), note.back.clone()],
        )
    } else {
        (
            vec!["Front".to_string(), "Back".to_string()],
            vec![note.front.clone(), note.back.clone()],
        )
    }
}

/// Parse section-delimited template into a map of section name → content.
fn parse_sections(text: &str) -> Result<std::collections::HashMap<&str, String>> {
    let section_re = Regex::new(r"(?m)^===\s+(.+?)\s+===\s*$").unwrap();
    let mut sections = std::collections::HashMap::new();
    let matches: Vec<_> = section_re.find_iter(text).collect();

    if matches.is_empty() {
        bail!("Template structure invalid — no section markers found");
    }

    for i in 0..matches.len() {
        let cap = section_re.captures(matches[i].as_str()).unwrap();
        let name = cap.get(1).unwrap().as_str();

        if name == "END" {
            break;
        }

        let start = matches[i].end();
        let end = if i + 1 < matches.len() {
            matches[i + 1].start()
        } else {
            text.len()
        };

        let content = &text[start..end];
        // Strip comment lines and leading/trailing newlines
        let cleaned: String = content
            .lines()
            .filter(|line| !line.starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n");

        sections.insert(name, cleaned.trim_start_matches('\n').to_string());
    }

    Ok(sections)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn basic_note() -> Note {
        Note {
            id: 12345,
            front: "What is Rust?".to_string(),
            back: "A systems programming language".to_string(),
            tags: vec!["programming".to_string(), "rust".to_string()],
            model_name: "Basic".to_string(),
        }
    }

    fn cloze_note() -> Note {
        Note {
            id: 67890,
            front: "The capital of {{c1::France}} is {{c2::Paris}}".to_string(),
            back: "Geography fact".to_string(),
            tags: vec!["geography".to_string()],
            model_name: "Cloze".to_string(),
        }
    }

    // T031: Renders correct template for Basic note
    #[test]
    fn given_basic_note_when_rendering_template_then_shows_front_back_tags() {
        let note = basic_note();
        let template = NoteTemplate::from_note(&note);
        let text = template.to_string();

        assert!(text.contains("=== FRONT ==="));
        assert!(text.contains("=== BACK ==="));
        assert!(text.contains("=== TAGS ==="));
        assert!(text.contains("What is Rust?"));
        assert!(text.contains("A systems programming language"));
        assert!(text.contains("programming rust"));
        assert!(text.contains("=== ID ==="));
        assert!(text.contains("12345"));
        assert!(text.contains("=== NOTE TYPE ==="));
        assert!(text.contains("Basic"));
    }

    // T032: Renders correct template for Cloze note
    #[test]
    fn given_cloze_note_when_rendering_template_then_shows_text_extra() {
        let note = cloze_note();
        let template = NoteTemplate::from_note(&note);
        let text = template.to_string();

        assert!(text.contains("=== TEXT ==="));
        assert!(text.contains("=== EXTRA ==="));
        assert!(text.contains("{{c1::France}}"));
        assert!(text.contains("Geography fact"));
    }

    // T033: Parses valid edited template back
    #[test]
    fn given_valid_template_when_parsing_then_returns_note_template() {
        let note = basic_note();
        let template = NoteTemplate::from_note(&note);
        let text = template.to_string();

        let parsed = NoteTemplate::from_string(&text, &note).unwrap();
        assert_eq!(parsed.note_id, 12345);
        assert_eq!(parsed.field_values[0], "What is Rust?");
        assert_eq!(parsed.field_values[1], "A systems programming language");
        assert_eq!(parsed.tags, vec!["programming", "rust"]);
    }

    // T034: Rejects template with missing section marker
    #[test]
    fn given_template_missing_section_when_parsing_then_error() {
        let note = basic_note();
        let text = "=== ID ===\n12345\n\n=== NOTE TYPE ===\nBasic\n\n=== FRONT ===\nQ\n\n=== TAGS ===\ntag\n\n=== END ===\n";

        let result = NoteTemplate::from_string(text, &note);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("BACK"));
    }

    // T035: Rejects template with empty required field
    #[test]
    fn given_template_with_empty_front_when_validating_then_error() {
        let note = basic_note();
        let template = NoteTemplate {
            note_id: 12345,
            note_type_name: "Basic".to_string(),
            field_names: vec!["Front".to_string(), "Back".to_string()],
            field_values: vec!["".to_string(), "Answer".to_string()],
            tags: vec![],
        };

        let result = template.validate(&note);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    // T036: Rejects Cloze without cloze deletion
    #[test]
    fn given_cloze_template_without_cloze_marker_when_validating_then_error() {
        let note = cloze_note();
        let template = NoteTemplate {
            note_id: 67890,
            note_type_name: "Cloze".to_string(),
            field_names: vec!["Text".to_string(), "Extra".to_string()],
            field_values: vec!["No cloze here".to_string(), "Extra".to_string()],
            tags: vec![],
        };

        let result = template.validate(&note);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cloze deletion"));
    }

    // T037: Preserves raw HTML
    #[test]
    fn given_html_content_when_roundtripping_then_preserved() {
        let note = Note {
            id: 111,
            front: "<b>Bold</b> and <i>italic</i>".to_string(),
            back: "<div>Answer</div>".to_string(),
            tags: vec![],
            model_name: "Basic".to_string(),
        };

        let template = NoteTemplate::from_note(&note);
        let text = template.to_string();
        let parsed = NoteTemplate::from_string(&text, &note).unwrap();

        assert_eq!(parsed.field_values[0], "<b>Bold</b> and <i>italic</i>");
        assert_eq!(parsed.field_values[1], "<div>Answer</div>");
    }

    // T038: Rejects note ID mismatch
    #[test]
    fn given_template_with_wrong_id_when_parsing_then_error() {
        let note = basic_note();
        let text = "=== ID ===\n99999\n\n=== NOTE TYPE ===\nBasic\n\n=== FRONT ===\nQ\n\n=== BACK ===\nA\n\n=== TAGS ===\ntag\n\n=== END ===\n";

        let result = NoteTemplate::from_string(&text, &note);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("mismatch"));
    }
}
