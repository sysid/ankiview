// src/util/text.rs
use html_escape::decode_html_entities;
use regex::Regex;

/// Extract the first line of plain text from HTML content.
///
/// This function:
/// 1. Decodes HTML entities (e.g., &amp; → &)
/// 2. Removes all HTML tags
/// 3. Extracts the first non-empty line
/// 4. Trims whitespace
///
/// # Examples
///
/// ```
/// use ankiview::util::text::extract_first_line;
///
/// let html = "<p>What is a Tree?</p><p>Second line</p>";
/// let first_line = extract_first_line(html);
/// assert_eq!(first_line, "What is a Tree?");
/// ```
pub fn extract_first_line(html: &str) -> String {
    // Decode HTML entities first
    let decoded = decode_html_entities(html).to_string();

    // Replace block-level HTML tags with newlines to preserve line breaks
    let block_re = Regex::new(r"</?(p|div|br|li|h[1-6])[^>]*>").unwrap();
    let with_newlines = block_re.replace_all(&decoded, "\n").into_owned();

    // Remove all remaining HTML tags
    let tag_re = Regex::new(r"<[^>]+>").unwrap();
    let no_tags = tag_re.replace_all(&with_newlines, "").into_owned();

    // Split by newlines and find first non-empty line
    no_tags
        .lines()
        .map(|line| line.trim())
        .find(|line| !line.is_empty())
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_simple_html_when_extracting_first_line_then_returns_text_without_tags() {
        let html = "<p>What is a Tree?</p>";
        assert_eq!(extract_first_line(html), "What is a Tree?");
    }

    #[test]
    fn given_multiline_html_when_extracting_first_line_then_returns_only_first_line() {
        let html = "<p>First line</p><p>Second line</p>";
        assert_eq!(extract_first_line(html), "First line");
    }

    #[test]
    fn given_html_entities_when_extracting_first_line_then_decodes_entities() {
        let html = "<p>Trees &amp; Graphs</p>";
        assert_eq!(extract_first_line(html), "Trees & Graphs");
    }

    #[test]
    fn given_nested_tags_when_extracting_first_line_then_removes_all_tags() {
        let html = "<div><strong>Bold</strong> and <em>italic</em></div>";
        assert_eq!(extract_first_line(html), "Bold and italic");
    }

    #[test]
    fn given_empty_html_when_extracting_first_line_then_returns_empty_string() {
        let html = "";
        assert_eq!(extract_first_line(html), "");
    }

    #[test]
    fn given_only_tags_when_extracting_first_line_then_returns_empty_string() {
        let html = "<div></div><p></p>";
        assert_eq!(extract_first_line(html), "");
    }

    #[test]
    fn given_whitespace_around_text_when_extracting_first_line_then_trims_whitespace() {
        let html = "<p>  What is a Tree?  </p>";
        assert_eq!(extract_first_line(html), "What is a Tree?");
    }

    #[test]
    fn given_line_breaks_in_html_when_extracting_first_line_then_handles_correctly() {
        let html = "<p>\nWhat is a Tree?\n</p><p>Second</p>";
        assert_eq!(extract_first_line(html), "What is a Tree?");
    }
}
