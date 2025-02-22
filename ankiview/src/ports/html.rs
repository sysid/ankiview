// src/ports/html.rs
use crate::domain::Note;
use html_escape::decode_html_entities;
use regex::Regex;
use tracing::instrument;

#[derive(Debug)]
pub struct HtmlPresenter;

impl HtmlPresenter {
    pub fn new() -> Self {
        Self
    }

    #[instrument(level = "debug", ret)]
    fn process_content(&self, content: &str) -> String {
        // First decode any HTML entities
        let decoded = decode_html_entities(&content).to_string();

        // Replace code blocks containing LaTeX with just the LaTeX content
        let code_block_re = Regex::new(
            r"<pre><code[^>]*>(?s)\s*((?:\$\$.*?\$\$)|(?:\$.*?\$))\s*</code></pre>"
        ).unwrap();

        code_block_re
            .replace_all(&decoded, |caps: &regex::Captures| {
                caps.get(1)
                    .map_or("", |m| m.as_str().trim())
                    .to_string()
            })
            .into_owned()
    }

    pub fn render(&self, note: &Note) -> String {
        let front = self.process_content(&note.front);
        let back = self.process_content(&note.back);
        let tags = note.tags.join(", ");

        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Anki Note {}</title>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/mathjax/3.2.2/es5/tex-mml-chtml.js"></script>
    <script>
        window.MathJax = {{
            tex: {{
                inlineMath: [['$', '$']],
                displayMath: [['$$', '$$']],
                processEscapes: true,
                packages: ['base', 'ams', 'noerrors', 'noundefined']
            }},
            options: {{
                processHtmlClass: 'tex2jax_process'
            }},
            startup: {{
                ready: () => {{
                    MathJax.startup.defaultReady();
                }}
            }}
        }};
    </script>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            line-height: 1.6;
            max-width: 800px;
            margin: 2rem auto;
            padding: 0 1rem;
            background-color: #f5f5f5;
        }}
        .card {{
            background: white;
            border-radius: 8px;
            padding: 2rem;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        pre {{
            white-space: pre-wrap;
            word-wrap: break-word;
            background-color: #f8f9fa;
            padding: 1rem;
            border-radius: 4px;
            overflow-x: auto;
        }}
        code {{
            background-color: #f0f0f0;
            padding: 2px 4px;
            border-radius: 3px;
            font-family: monospace;
        }}
        .card-front {{
            margin-bottom: 2rem;
            padding-bottom: 1rem;
            border-bottom: 2px solid #eee;
        }}
        .note-info {{
            margin-top: 1rem;
            padding-top: 1rem;
            border-top: 1px solid #eee;
            font-size: 0.9em;
            color: #666;
        }}
        .tags {{
            margin-top: 0.5rem;
        }}
        .tag {{
            display: inline-block;
            background: #e9ecef;
            padding: 2px 8px;
            border-radius: 4px;
            margin-right: 4px;
            font-size: 0.8em;
        }}
        .tex2jax_process {{
            margin: 1em 0;
        }}
    </style>
</head>
<body>
    <div class="card">
        <div class="card-front">
            <h2>Question</h2>
            <div class="tex2jax_process">{front}</div>
        </div>
        <div class="card-back">
            <h2>Answer</h2>
            <div class="tex2jax_process">{back}</div>
        </div>
        <div class="note-info">
            <div>Note ID: {note_id}</div>
            <div>Model: {model}</div>
            <div class="tags">
                Tags: {tags}
            </div>
        </div>
    </div>
</body>
</html>"#,
            note.id,
            front = front,
            back = back,
            note_id = note.id,
            model = note.model_name,
            tags = if tags.is_empty() {
                "No tags".to_string()
            } else {
                tags
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Note;
    use rstest::rstest;

    #[rstest]
    #[case(
        r#"<pre><code class="language-tex">$$\begin{cases}
a, & x < 0 \\
b, & x \geq 0
\end{cases}$$</code></pre>"#,
        r"$$\begin{cases}
a, & x < 0 \\
b, & x \geq 0
\end{cases}$$"
    )]
    #[case(
        "Simple inline $x^2$ math",
        "Simple inline $x^2$ math"
    )]
    fn test_latex_extraction(#[case] input: &str, #[case] expected: &str) {
        let presenter = HtmlPresenter::new();
        let note = Note {
            id: 1,
            front: input.to_string(),
            back: "Test".to_string(),
            tags: vec![],
            model_name: "Basic".to_string(),
        };

        let processed = presenter.process_content(&input);
        assert_eq!(processed, expected);
    }
}