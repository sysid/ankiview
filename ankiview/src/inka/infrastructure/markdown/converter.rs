use lazy_static::lazy_static;
use pulldown_cmark::{html, Options, Parser};
use regex::Regex;

lazy_static! {
    static ref NEWLINE_TAG_REGEX: Regex =
        Regex::new(r"\n?(<.+?>)\n?").expect("Failed to compile newline tag regex");
    static ref INLINE_MATH_REGEX: Regex =
        Regex::new(r"\$([^\s$][^$]*[^\s$])\$").expect("Failed to compile inline math regex");
    // Match $$ blocks in HTML context (may have newlines and whitespace)
    static ref BLOCK_MATH_REGEX: Regex =
        Regex::new(r"\$\$\s*((?:.|\n)+?)\s*\$\$").expect("Failed to compile block math regex");
}

pub fn markdown_to_html(text: &str) -> String {
    // Parse markdown with pulldown-cmark first
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(text, options);

    // Convert events to HTML
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    // Post-process: Convert math delimiters and remove newlines around tags
    let html_output = convert_math_delimiters(&html_output);
    remove_newlines_around_tags(&html_output)
}

/// Convert $ and $$ delimiters to MathJax format after HTML rendering
fn convert_math_delimiters(html: &str) -> String {
    // First handle block math ($$...$$) to avoid conflicts with inline
    let html = BLOCK_MATH_REGEX.replace_all(html, r"\[$1\]");

    // Then handle inline math ($...$)
    INLINE_MATH_REGEX
        .replace_all(&html, r"\($1\)")
        .to_string()
}

fn remove_newlines_around_tags(html: &str) -> String {
    NEWLINE_TAG_REGEX.replace_all(html, "$1").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_markdown_text_when_converting_then_renders_html() {
        let input = "**bold** and *italic*";
        let html = markdown_to_html(input);

        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<em>italic</em>"));
    }

    #[test]
    fn given_markdown_with_newlines_around_tags_when_converting_then_removes_them() {
        // Anki quirk: newlines around HTML tags render as visible breaks
        let input = "Text\n\n**bold**\n\nMore";
        let html = markdown_to_html(input);

        // Should not have \n<strong> or </strong>\n
        assert!(!html.contains("\n<"));
        assert!(!html.contains(">\n"));
    }

    #[test]
    fn given_markdown_with_math_when_converting_then_uses_mathjax_delimiters() {
        let input = "Inline $f(x)$ and block:\n$$\ng(x)\n$$";
        let html = markdown_to_html(input);

        assert!(html.contains(r"\(f(x)\)"));
        assert!(html.contains(r"\[g(x)\]"));
    }

    #[test]
    fn given_complex_math_when_converting_then_preserves_latex() {
        let input = r"$$
\sum_{i=1}^{n} i = \frac{n(n+1)}{2}
$$";
        let html = markdown_to_html(input);

        assert!(html.contains(r"\[\sum_{i=1}^{n}"));
    }

    #[test]
    fn given_code_block_when_converting_then_preserves_for_highlightjs() {
        let input = "```rust\nfn main() {}\n```";
        let html = markdown_to_html(input);

        // Should output highlight.js-compatible structure
        assert!(html.contains("<pre><code class=\"language-rust\">"));
        assert!(html.contains("fn main()"));
        assert!(html.contains("</code></pre>"));
    }

    #[test]
    fn given_inline_code_when_converting_then_wraps_in_code_tag() {
        let input = "This is `inline code` example";
        let html = markdown_to_html(input);

        assert!(html.contains("<code>inline code</code>"));
    }

    #[test]
    fn given_sql_code_block_when_converting_then_uses_language_class() {
        let input = "```sql\nSELECT * FROM users WHERE id = 1;\n```";
        let html = markdown_to_html(input);

        assert!(html.contains("<pre><code class=\"language-sql\">"));
        assert!(html.contains("SELECT * FROM users WHERE id = 1;"));
        assert!(html.contains("</code></pre>"));
    }

    #[test]
    fn given_python_code_block_when_converting_then_uses_language_class() {
        let input = "```python\nimport model\n\ndef start_mappers():\n    pass\n```";
        let html = markdown_to_html(input);

        assert!(html.contains("<pre><code class=\"language-python\">"));
        assert!(html.contains("import model"));
        assert!(html.contains("def start_mappers():"));
        assert!(html.contains("</code></pre>"));
    }

    #[test]
    fn given_unlabeled_code_block_when_converting_then_still_wraps_properly() {
        let input = "```\ngeneric code block\n```";
        let html = markdown_to_html(input);

        assert!(html.contains("<pre><code>"));
        assert!(html.contains("generic code block"));
        assert!(html.contains("</code></pre>"));
    }
}
