use regex::Regex;
use lazy_static::lazy_static;

pub struct SectionParser {
    section_regex: Regex,
}

impl SectionParser {
    pub fn new() -> Self {
        // Regex pattern: ^---\n(.+?)^---$
        // Multiline and dotall flags
        let section_regex = Regex::new(r"(?ms)^---\n(.+?)^---$")
            .expect("Failed to compile section regex");

        Self { section_regex }
    }

    pub fn parse<'a>(&self, input: &'a str) -> Vec<&'a str> {
        self.section_regex
            .captures_iter(input)
            .filter_map(|cap| cap.get(1))
            .map(|m| m.as_str())
            .collect()
    }
}

impl Default for SectionParser {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static! {
    static ref DECK_REGEX: Regex = Regex::new(r"(?m)^Deck:\s*(.+?)$")
        .expect("Failed to compile deck regex");

    static ref TAGS_REGEX: Regex = Regex::new(r"(?m)^Tags:\s*(.+?)$")
        .expect("Failed to compile tags regex");

    static ref NOTE_START_REGEX: Regex = Regex::new(r"(?m)^(?:<!--ID:\S+-->\n)?^\d+\.")
        .expect("Failed to compile note start regex");
}

pub fn extract_deck_name(section: &str) -> Option<String> {
    DECK_REGEX
        .captures(section)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().trim().to_string())
}

pub fn extract_tags(section: &str) -> Vec<String> {
    TAGS_REGEX
        .captures(section)
        .and_then(|cap| cap.get(1))
        .map(|m| {
            m.as_str()
                .split_whitespace()
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default()
}

pub fn extract_note_strings(section: &str) -> Vec<String> {
    // Find all positions where notes start (either "1. " or "<!--ID:...-->\n1. ")
    let mut note_positions: Vec<usize> = Vec::new();

    // Find all lines starting with digits followed by a dot
    for line in section.lines() {
        if let Some(trimmed) = line.trim_start().strip_prefix(|c: char| c.is_ascii_digit()) {
            if trimmed.starts_with('.') {
                // Found a note start, get its position in the original string
                if let Some(pos) = section.find(line) {
                    // Check if there's an ID comment before this line
                    let before = &section[..pos];
                    if let Some(last_line) = before.lines().last() {
                        if last_line.trim().starts_with("<!--ID:") {
                            // Include the ID comment
                            if let Some(id_pos) = section[..pos].rfind("<!--ID:") {
                                note_positions.push(id_pos);
                                continue;
                            }
                        }
                    }
                    note_positions.push(pos);
                }
            }
        }
    }

    // Extract note strings by slicing between positions
    let mut notes = Vec::new();
    for i in 0..note_positions.len() {
        let start = note_positions[i];
        let end = if i + 1 < note_positions.len() {
            note_positions[i + 1]
        } else {
            section.len()
        };

        let note_str = section[start..end].trim_end().to_string();
        notes.push(note_str);
    }

    notes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_markdown_with_section_when_parsing_then_finds_section() {
        let input = r#"# Heading

---
Deck: Test

1. Question 1
> Answer 1

2. Question 2
> Answer 2
---

More text after"#;

        let sections = SectionParser::new().parse(input);

        assert_eq!(sections.len(), 1);
        assert!(sections[0].contains("Deck: Test"));
        assert!(sections[0].contains("Question 1"));
    }

    #[test]
    fn given_markdown_with_multiple_sections_when_parsing_then_finds_all() {
        let input = r#"---
Deck: First
1. Q1
> A1
---

Text between

---
Deck: Second
1. Q2
> A2
---"#;

        let sections = SectionParser::new().parse(input);

        assert_eq!(sections.len(), 2);
        assert!(sections[0].contains("First"));
        assert!(sections[1].contains("Second"));
    }

    #[test]
    fn given_markdown_without_sections_when_parsing_then_returns_empty() {
        let input = "Just regular markdown\nNo sections here";
        let sections = SectionParser::new().parse(input);

        assert_eq!(sections.len(), 0);
    }

    #[test]
    fn given_section_with_deck_when_extracting_then_returns_deck_name() {
        let section = "Deck: MyDeck\nTags: tag1\n1. Question\n> Answer";
        let deck = extract_deck_name(section);

        assert_eq!(deck, Some("MyDeck".to_string()));
    }

    #[test]
    fn given_section_without_deck_when_extracting_then_returns_none() {
        let section = "Tags: tag1\n1. Question\n> Answer";
        let deck = extract_deck_name(section);

        assert_eq!(deck, None);
    }

    #[test]
    fn given_section_with_deck_and_whitespace_when_extracting_then_trims() {
        let section = "Deck:   MyDeck   \n1. Q";
        let deck = extract_deck_name(section);

        assert_eq!(deck, Some("MyDeck".to_string()));
    }

    #[test]
    fn given_section_with_tags_when_extracting_then_returns_tag_vec() {
        let section = "Deck: MyDeck\nTags: tag1 tag2 tag3\n";
        let tags = extract_tags(section);

        assert_eq!(tags, vec!["tag1", "tag2", "tag3"]);
    }

    #[test]
    fn given_section_without_tags_when_extracting_then_returns_empty() {
        let section = "Deck: MyDeck\n1. Q";
        let tags = extract_tags(section);

        assert_eq!(tags, Vec::<String>::new());
    }

    #[test]
    fn given_section_with_empty_tags_when_extracting_then_returns_empty() {
        let section = "Tags:   \n1. Q";
        let tags = extract_tags(section);

        assert_eq!(tags, Vec::<String>::new());
    }

    #[test]
    fn given_section_with_two_notes_when_extracting_then_returns_two_strings() {
        let section = "Deck: Test\n1. First Q\n> First A\n2. Second Q\n> Second A";
        let notes = extract_note_strings(section);

        assert_eq!(notes.len(), 2);
        assert!(notes[0].contains("First Q"));
        assert!(notes[1].contains("Second Q"));
    }

    #[test]
    fn given_section_with_id_comments_when_extracting_then_includes_ids() {
        let section = "Deck: Test\n<!--ID:123-->\n1. Q1\n> A1\n<!--ID:456-->\n2. Q2\n> A2";
        let notes = extract_note_strings(section);

        assert_eq!(notes.len(), 2);
        assert!(notes[0].contains("<!--ID:123-->"));
        assert!(notes[1].contains("<!--ID:456-->"));
    }

    #[test]
    fn given_section_with_cloze_and_basic_when_extracting_then_finds_both() {
        let section = "1. Basic Q\n> Basic A\n2. Cloze {{c1::text}}";
        let notes = extract_note_strings(section);

        assert_eq!(notes.len(), 2);
    }
}
