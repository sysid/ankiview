use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref ANKI_CLOZE_REGEX: Regex = Regex::new(r"\{\{c\d+::[\s\S]*?\}\}")
        .expect("Failed to compile Anki cloze regex");

    static ref EXPLICIT_SHORT_CLOZE_REGEX: Regex = Regex::new(r"\{c?(\d+)::([\s\S]*?)\}")
        .expect("Failed to compile explicit short cloze regex");

    static ref IMPLICIT_SHORT_CLOZE_REGEX: Regex = Regex::new(r"\{([\s\S]*?)\}")
        .expect("Failed to compile implicit short cloze regex");

    static ref CODE_BLOCK_REGEX: Regex = Regex::new(r"```[\s\S]+?```")
        .expect("Failed to compile code block regex");

    static ref INLINE_CODE_REGEX: Regex = Regex::new(r"`[\S\s]+?`")
        .expect("Failed to compile inline code regex");

    static ref BLOCK_MATH_REGEX: Regex = Regex::new(r"\$\$[\s\S]+?\$\$")
        .expect("Failed to compile block math regex");

    static ref INLINE_MATH_REGEX: Regex = Regex::new(r"\$[^\s$][^$]*?\$")
        .expect("Failed to compile inline math regex");
}

pub fn is_anki_cloze(text: &str) -> bool {
    ANKI_CLOZE_REGEX.is_match(text)
}

pub fn convert_cloze_syntax(text: &str) -> String {
    // Protect code and math blocks
    let (text, code_blocks) = protect_code_blocks(text);
    let (text, math_blocks) = protect_math_blocks(&text);

    // Find all cloze-like patterns
    let mut result = text.clone();
    let mut counter = 1;

    // Process each potential cloze deletion
    let all_clozes: Vec<_> = find_all_clozes(&text);

    for cloze in all_clozes {
        if is_anki_cloze(&cloze) {
            // Already in Anki format, skip
            continue;
        }

        // Try explicit short syntax: {1::text} or {c1::text}
        if let Some(caps) = EXPLICIT_SHORT_CLOZE_REGEX.captures(&cloze) {
            let index = caps.get(1).unwrap().as_str();
            let content = caps.get(2).unwrap().as_str();
            let replacement = format!("{{{{c{}::{}}}}}", index, content);
            result = result.replacen(&cloze, &replacement, 1);
            continue;
        }

        // Try implicit short syntax: {text}
        if let Some(caps) = IMPLICIT_SHORT_CLOZE_REGEX.captures(&cloze) {
            let content = caps.get(1).unwrap().as_str();
            let replacement = format!("{{{{c{}::{}}}}}", counter, content);
            result = result.replacen(&cloze, &replacement, 1);
            counter += 1;
        }
    }

    // Restore protected blocks
    let result = restore_math_blocks(&result, math_blocks);
    let result = restore_code_blocks(&result, code_blocks);

    result
}

fn find_all_clozes(text: &str) -> Vec<String> {
    // Find all {...} patterns that aren't already {{c...}}
    let mut clozes = Vec::new();
    let mut chars = text.chars().peekable();
    let mut current = String::new();
    let mut in_cloze = false;
    let mut brace_count = 0;

    while let Some(c) = chars.next() {
        if c == '{' {
            if chars.peek() == Some(&'{') {
                // Skip Anki format
                current.push(c);
                current.push(chars.next().unwrap());
                continue;
            }
            in_cloze = true;
            brace_count = 1;
            current.push(c);
        } else if c == '}' && in_cloze {
            current.push(c);
            brace_count -= 1;
            if brace_count == 0 {
                clozes.push(current.clone());
                current.clear();
                in_cloze = false;
            }
        } else if in_cloze {
            current.push(c);
            if c == '{' {
                brace_count += 1;
            }
        }
    }

    clozes
}

fn protect_code_blocks(text: &str) -> (String, Vec<String>) {
    let mut blocks = Vec::new();
    let mut result = text.to_string();

    // Block code first (must come before inline)
    for mat in CODE_BLOCK_REGEX.find_iter(text) {
        blocks.push(mat.as_str().to_string());
    }
    result = CODE_BLOCK_REGEX.replace_all(&result, "___CODE_BLOCK___").to_string();

    // Inline code
    for mat in INLINE_CODE_REGEX.find_iter(&result) {
        blocks.push(mat.as_str().to_string());
    }
    result = INLINE_CODE_REGEX.replace_all(&result, "___INLINE_CODE___").to_string();

    (result, blocks)
}

fn protect_math_blocks(text: &str) -> (String, Vec<String>) {
    let mut blocks = Vec::new();
    let mut result = text.to_string();

    // Block math first (MUST come before inline to avoid matching $$ as two $ markers)
    for mat in BLOCK_MATH_REGEX.find_iter(text) {
        blocks.push(mat.as_str().to_string());
    }
    result = BLOCK_MATH_REGEX.replace_all(&result, "___MATH_BLOCK___").to_string();

    // Inline math - now the $$ are already protected
    for mat in INLINE_MATH_REGEX.find_iter(&result) {
        blocks.push(mat.as_str().to_string());
    }
    result = INLINE_MATH_REGEX.replace_all(&result, "___INLINE_MATH___").to_string();

    (result, blocks)
}

fn restore_code_blocks(text: &str, blocks: Vec<String>) -> String {
    let mut result = text.to_string();
    for block in blocks {
        if result.contains("___CODE_BLOCK___") {
            result = result.replacen("___CODE_BLOCK___", &block, 1);
        } else if result.contains("___INLINE_CODE___") {
            result = result.replacen("___INLINE_CODE___", &block, 1);
        }
    }
    result
}

fn restore_math_blocks(text: &str, blocks: Vec<String>) -> String {
    let mut result = text.to_string();
    for block in blocks {
        if result.contains("___MATH_BLOCK___") {
            result = result.replacen("___MATH_BLOCK___", &block, 1);
        } else if result.contains("___INLINE_MATH___") {
            result = result.replacen("___INLINE_MATH___", &block, 1);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_anki_format_cloze_when_checking_then_returns_true() {
        assert!(is_anki_cloze("{{c1::text}}"));
        assert!(is_anki_cloze("{{c12::multiple words}}"));
        assert!(!is_anki_cloze("{1::text}"));
        assert!(!is_anki_cloze("{text}"));
    }

    #[test]
    fn given_explicit_short_cloze_when_converting_then_transforms_to_anki() {
        let input = "Text {1::hidden} more {c2::also}";
        let output = convert_cloze_syntax(input);

        assert_eq!(output, "Text {{c1::hidden}} more {{c2::also}}");
    }

    #[test]
    fn given_already_anki_format_when_converting_then_unchanged() {
        let input = "Text {{c1::already}} correct";
        let output = convert_cloze_syntax(input);

        assert_eq!(output, "Text {{c1::already}} correct");
    }

    #[test]
    fn given_implicit_short_cloze_when_converting_then_numbers_sequentially() {
        let input = "First {one} then {two} finally {three}";
        let output = convert_cloze_syntax(input);

        assert_eq!(output, "First {{c1::one}} then {{c2::two}} finally {{c3::three}}");
    }

    #[test]
    fn given_cloze_with_code_block_when_converting_then_preserves_code() {
        let input = "Text {answer}\n```\n{not_a_cloze}\n```";
        let output = convert_cloze_syntax(input);

        assert_eq!(output, "Text {{c1::answer}}\n```\n{not_a_cloze}\n```");
    }

    #[test]
    fn given_cloze_with_inline_code_when_converting_then_preserves_code() {
        let input = "Text {answer} and `code {with braces}`";
        let output = convert_cloze_syntax(input);

        assert!(output.contains("{{c1::answer}}"));
        assert!(output.contains("`code {with braces}`"));
    }

    #[test]
    fn given_cloze_with_math_when_converting_then_preserves_math() {
        let input = "Equation {answer} is $$x^{2}$$ and inline $y^{3}$";
        let output = convert_cloze_syntax(input);

        assert_eq!(output, "Equation {{c1::answer}} is $$x^{2}$$ and inline $y^{3}$");
    }
}
