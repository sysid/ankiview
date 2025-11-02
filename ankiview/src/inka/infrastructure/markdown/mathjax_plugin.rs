use markdown_it::parser::block::{BlockRule, BlockState};
use markdown_it::parser::inline::{InlineRule, InlineState};
use markdown_it::{MarkdownIt, Node, NodeValue, Renderer};

#[derive(Debug)]
pub struct InlineMath {
    pub content: String,
}

impl NodeValue for InlineMath {
    fn render(&self, _node: &Node, fmt: &mut dyn Renderer) {
        // Render as \(...\) for MathJax
        fmt.text(&format!(r"\({}\)", self.content));
    }
}

struct InlineMathScanner;

impl InlineRule for InlineMathScanner {
    const MARKER: char = '$';

    fn run(state: &mut InlineState) -> Option<(Node, usize)> {
        let input = &state.src[state.pos..state.pos_max];

        // Check if we start with $
        if !input.starts_with('$') {
            return None;
        }

        // Don't match if $ is followed by whitespace
        if input.len() < 2 || input.chars().nth(1)?.is_whitespace() {
            return None;
        }

        // Find the closing $
        let mut end_pos = None;
        let chars: Vec<char> = input.chars().collect();

        for i in 1..chars.len() {
            if chars[i] == '$' {
                // Don't match if $ is preceded by whitespace
                if i > 0 && !chars[i - 1].is_whitespace() {
                    end_pos = Some(i);
                    break;
                }
            }
        }

        if let Some(end) = end_pos {
            // Extract content between the $...$ (excluding the $ markers)
            let content: String = chars[1..end].iter().collect();
            let match_len = end + 1; // Include both $ markers

            let node = Node::new(InlineMath { content });
            return Some((node, match_len));
        }

        None
    }
}

#[derive(Debug)]
pub struct BlockMath {
    pub content: String,
}

impl NodeValue for BlockMath {
    fn render(&self, _node: &Node, fmt: &mut dyn Renderer) {
        // Render as \[...\] for MathJax
        fmt.text(&format!(r"\[{}\]", self.content));
    }
}

struct BlockMathScanner;

impl BlockRule for BlockMathScanner {
    fn run(state: &mut BlockState) -> Option<(Node, usize)> {
        // Get the current line
        if state.line >= state.line_max {
            return None;
        }

        let start_line = state.line;
        let line = state.get_line(start_line);

        // Check if line starts with $$
        if !line.trim().starts_with("$$") {
            return None;
        }

        // Find the closing $$
        let mut end_line = None;
        for line_num in (start_line + 1)..state.line_max {
            let line = state.get_line(line_num);
            if line.trim().starts_with("$$") {
                end_line = Some(line_num);
                break;
            }
        }

        if let Some(end) = end_line {
            // Extract content between the $$ markers
            let mut content_lines = Vec::new();
            for line_num in (start_line + 1)..end {
                content_lines.push(state.get_line(line_num).to_string());
            }
            let content = content_lines.join("\n");

            let node = Node::new(BlockMath { content });
            // Return the closing $$ line - the parser will advance past it
            return Some((node, end));
        }

        None
    }
}

pub fn add_mathjax_plugin(md: &mut MarkdownIt) {
    md.inline.add_rule::<InlineMathScanner>();
    md.block.add_rule::<BlockMathScanner>().before_all();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_inline_math_when_parsing_then_creates_math_token() {
        let input = "This is $f(x) = x^2$ inline math";
        let mut parser = MarkdownIt::new();
        markdown_it::plugins::cmark::add(&mut parser);
        add_mathjax_plugin(&mut parser);

        let ast = parser.parse(input);
        let html = ast.render();

        // Should render with MathJax delimiters
        assert!(html.contains(r"\(f(x) = x^2\)"));
    }

    #[test]
    fn given_block_math_when_parsing_then_creates_block_math_token() {
        let input = "$$\nf(x) = \\int_0^1 x^2 dx\n$$";
        let mut parser = MarkdownIt::new();
        markdown_it::plugins::cmark::add(&mut parser);
        add_mathjax_plugin(&mut parser);

        let html = parser.parse(input).render();

        assert!(html.contains(r"\[f(x) = \int_0^1 x^2 dx\]"));
    }

    #[test]
    fn given_mixed_math_when_parsing_then_handles_both_types() {
        let input = r#"Inline $a=b$ and block:

$$
\sum_{i=1}^n i = \frac{n(n+1)}{2}
$$

More text."#;

        let mut parser = MarkdownIt::new();
        markdown_it::plugins::cmark::add(&mut parser);
        add_mathjax_plugin(&mut parser);

        let html = parser.parse(input).render();

        assert!(html.contains(r"\(a=b\)"));
        assert!(html.contains(r"\[\sum_{i=1}^n i = \frac{n(n+1)}{2}\]"));
    }
}
