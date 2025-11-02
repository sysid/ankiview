use super::mathjax_plugin::add_mathjax_plugin;
use lazy_static::lazy_static;
use markdown_it::MarkdownIt;
use regex::Regex;

lazy_static! {
    static ref NEWLINE_TAG_REGEX: Regex =
        Regex::new(r"\n?(<.+?>)\n?").expect("Failed to compile newline tag regex");
}

pub fn markdown_to_html(text: &str) -> String {
    let mut parser = MarkdownIt::new();
    markdown_it::plugins::cmark::add(&mut parser);
    markdown_it::plugins::extra::add(&mut parser);
    add_mathjax_plugin(&mut parser);

    let html = parser.parse(text).render();

    // Remove newlines around HTML tags (Anki rendering quirk)
    remove_newlines_around_tags(&html)
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

        // markdown-it extra plugin uses syntect for syntax highlighting with inline styles
        assert!(html.contains("<pre"));
        assert!(html.contains("fn"));
        assert!(html.contains("main"));
    }

    #[test]
    fn given_inline_code_when_converting_then_wraps_in_code_tag() {
        let input = "This is `inline code` example";
        let html = markdown_to_html(input);

        assert!(html.contains("<code>inline code</code>"));
    }
}
