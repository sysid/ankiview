use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref BASIC_CARD_REGEX: Regex =
        Regex::new(r"(?m)(?:^<!--ID:\S+-->\n)?^\d+\.[\s\S]+?(?:^>.*?(?:\n|$))+")
            .expect("Failed to compile basic card regex");
    static ref ID_REGEX: Regex =
        Regex::new(r"(?m)^<!--ID:(\S+)-->$").expect("Failed to compile ID regex");
}

pub fn is_basic_card(note_str: &str) -> bool {
    BASIC_CARD_REGEX.is_match(note_str)
}

pub fn is_cloze_card(note_str: &str) -> bool {
    // A cloze card has curly braces (for cloze deletions)
    // and doesn't have the answer marker (>)
    note_str.contains('{')
        && !note_str
            .lines()
            .any(|line| line.trim_start().starts_with('>'))
}

pub fn parse_basic_card_fields(note_str: &str) -> Result<(String, String)> {
    // Find the first line with a number and dot
    let lines: Vec<&str> = note_str.lines().collect();
    let mut question_lines = Vec::new();
    let mut answer_lines = Vec::new();
    let mut in_answer = false;

    for line in lines {
        let trimmed = line.trim();

        // Skip ID comments
        if trimmed.starts_with("<!--ID:") {
            continue;
        }

        // Check if this is the start of an answer
        if trimmed.starts_with('>') {
            in_answer = true;
            answer_lines.push(line);
        } else if in_answer {
            // Once we're in answer mode, keep collecting
            answer_lines.push(line);
        } else {
            // We're in the question
            question_lines.push(line);
        }
    }

    // Extract the question text (remove the "1. " prefix)
    let front = question_lines.join("\n");
    let front = if let Some(stripped) = front.trim().strip_prefix(|c: char| c.is_ascii_digit()) {
        if let Some(after_dot) = stripped.strip_prefix('.') {
            after_dot.trim().to_string()
        } else {
            front
        }
    } else {
        front
    };

    if front.is_empty() {
        anyhow::bail!("Failed to extract question from basic card");
    }

    // Clean the answer
    let answer_text = answer_lines.join("\n");
    if answer_text.trim().is_empty() {
        anyhow::bail!("Failed to extract answer from basic card");
    }

    let back = clean_answer(&answer_text);

    Ok((front, back))
}

fn clean_answer(answer_raw: &str) -> String {
    answer_raw
        .lines()
        .map(|line| {
            // Remove '>' and first space/tab after it
            if line.len() > 1 && line.starts_with('>') {
                let without_prefix = &line[1..];
                if without_prefix.starts_with(' ') || without_prefix.starts_with('\t') {
                    &without_prefix[1..]
                } else {
                    without_prefix
                }
            } else if let Some(stripped) = line.strip_prefix('>') {
                stripped
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn parse_cloze_card_field(note_str: &str) -> Result<String> {
    // Similar to basic card, but just extract the text after the number
    let lines: Vec<&str> = note_str.lines().collect();
    let mut text_lines = Vec::new();

    for line in lines {
        let trimmed = line.trim();

        // Skip ID comments
        if trimmed.starts_with("<!--ID:") {
            continue;
        }

        text_lines.push(line);
    }

    // Extract the text (remove the "1. " prefix)
    let text = text_lines.join("\n");
    let text = if let Some(stripped) = text.trim().strip_prefix(|c: char| c.is_ascii_digit()) {
        if let Some(after_dot) = stripped.strip_prefix('.') {
            after_dot.trim().to_string()
        } else {
            text
        }
    } else {
        text
    };

    if text.is_empty() {
        anyhow::bail!("Failed to extract text from cloze card");
    }

    Ok(text)
}

pub fn extract_anki_id(note_str: &str) -> Option<i64> {
    ID_REGEX
        .captures(note_str)
        .and_then(|cap| cap.get(1))
        .and_then(|m| m.as_str().parse::<i64>().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_note_with_answer_when_checking_type_then_is_basic() {
        let note_str = "1. Question?\n> Answer!";

        assert!(is_basic_card(note_str));
        assert!(!is_cloze_card(note_str));
    }

    #[test]
    fn given_note_with_multiline_answer_when_checking_then_is_basic() {
        let note_str = "1. Q\n> Line 1\n> Line 2\n> Line 3";

        assert!(is_basic_card(note_str));
    }

    #[test]
    fn given_note_without_answer_when_checking_then_not_basic() {
        let note_str = "1. Just a question?";

        assert!(!is_basic_card(note_str));
    }

    #[test]
    fn given_basic_note_string_when_parsing_then_extracts_front_and_back() {
        let note_str = "1. What is 2+2?\n> It's 4";
        let (front, back) = parse_basic_card_fields(note_str).unwrap();

        assert_eq!(front, "What is 2+2?");
        assert_eq!(back, "It's 4");
    }

    #[test]
    fn given_basic_with_multiline_when_parsing_then_preserves_lines() {
        let note_str = "1. Multi\nline\nquestion\n> Multi\n> line\n> answer";
        let (front, back) = parse_basic_card_fields(note_str).unwrap();

        assert_eq!(front, "Multi\nline\nquestion");
        assert_eq!(back, "Multi\nline\nanswer");
    }

    #[test]
    fn given_basic_with_id_when_parsing_then_extracts_without_id() {
        let note_str = "<!--ID:123456-->\n1. Question\n> Answer";
        let (front, back) = parse_basic_card_fields(note_str).unwrap();

        assert_eq!(front, "Question");
        assert_eq!(back, "Answer");
    }

    #[test]
    fn given_note_without_answer_when_parsing_then_returns_error() {
        let note_str = "1. Only question";
        let result = parse_basic_card_fields(note_str);

        assert!(result.is_err());
    }

    #[test]
    fn given_cloze_note_string_when_parsing_then_extracts_text() {
        let note_str = "1. Paris is the {{c1::capital}} of {{c2::France}}";
        let text = parse_cloze_card_field(note_str).unwrap();

        assert_eq!(text, "Paris is the {{c1::capital}} of {{c2::France}}");
    }

    #[test]
    fn given_cloze_with_id_when_parsing_then_excludes_id() {
        let note_str = "<!--ID:999-->\n1. Text {{c1::cloze}}";
        let text = parse_cloze_card_field(note_str).unwrap();

        assert_eq!(text, "Text {{c1::cloze}}");
    }

    #[test]
    fn given_cloze_with_short_syntax_when_parsing_then_extracts() {
        let note_str = "1. Capital is {Paris}";
        let text = parse_cloze_card_field(note_str).unwrap();

        assert_eq!(text, "Capital is {Paris}");
    }

    #[test]
    fn given_note_with_id_when_parsing_then_extracts_id() {
        let note_str = "<!--ID:1234567890-->\n1. Question?";
        let id = extract_anki_id(note_str);

        assert_eq!(id, Some(1234567890));
    }

    #[test]
    fn given_note_without_id_when_parsing_then_returns_none() {
        let note_str = "1. Question?";
        let id = extract_anki_id(note_str);

        assert_eq!(id, None);
    }

    #[test]
    fn given_note_with_invalid_id_when_parsing_then_returns_none() {
        let note_str = "<!--ID:not_a_number-->\n1. Q";
        let id = extract_anki_id(note_str);

        assert_eq!(id, None);
    }
}
