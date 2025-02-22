// src/ports/html.rs
use crate::domain::Note;
use html_escape::decode_html_entities;
use regex::Regex;
use std::path::Path;
use tracing::instrument;

#[derive(Debug)]
pub struct HtmlPresenter {
    media_dir: Option<String>,
}

impl HtmlPresenter {
    pub fn new() -> Self {
        Self {
            media_dir: None
        }
    }

    pub fn with_media_dir<P: AsRef<Path>>(media_dir: P) -> Self {
        Self {
            media_dir: Some(media_dir.as_ref().to_string_lossy().into_owned())
        }
    }

    #[instrument(level = "debug", ret)]
    fn process_content(&self, content: &str) -> String {
        // First decode any HTML entities
        let decoded = decode_html_entities(&content).to_string();

        // Replace code blocks containing LaTeX with just the LaTeX content
        let code_block_re = Regex::new(
            r"<pre><code[^>]*>(?s)\s*((?:\$\$.*?\$\$)|(?:\$.*?\$))\s*</code></pre>"
        ).unwrap();

        let processed = code_block_re
            .replace_all(&decoded, |caps: &regex::Captures| {
                caps.get(1)
                    .map_or("", |m| m.as_str().trim())
                    .to_string()
            })
            .into_owned();

        // Handle image tags if media directory is set
        if let Some(ref media_dir) = self.media_dir {
            let img_re = Regex::new(r#"<img\s+src="([^"]+)"([^>]*)>"#).unwrap();
            img_re.replace_all(&processed, |caps: &regex::Captures| {
                let src = caps.get(1).unwrap().as_str();
                let attrs = caps.get(2).map_or("", |m| m.as_str());

                // If src is a URL, leave it unchanged
                if src.starts_with("http://") || src.starts_with("https://") {
                    format!(r#"<img src="{src}"{attrs}>"#)
                } else {
                    // Otherwise, prefix with media directory
                    format!(r#"<img src="file://{media_dir}/{src}"{attrs}>"#)
                }
            }).into_owned()
        } else {
            processed
        }
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
        img {{
            max-width: 100%;
            height: auto;
            display: block;
            margin: 1rem auto;
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
    use rstest::rstest;

    #[rstest]
    #[case(
        r#"<pre><code class="language-tex">$\begin{cases} a, & x < 0 \\ b, & x \geq 0 \end{cases}$</code></pre>"#,
        r"$\begin{cases} a, & x < 0 \\ b, & x \geq 0 \end{cases}$",
        Some("/media")
    )]
    #[case(
        r#"<img src="test.jpg" alt="test">"#,
        r#"<img src="file:///media/test.jpg" alt="test">"#,
        Some("/media")
    )]
    #[case(
        r#"<img src="https://example.com/test.jpg" alt="test">"#,
        r#"<img src="https://example.com/test.jpg" alt="test">"#,
        Some("/media")
    )]
    fn test_content_processing(
        #[case] input: &str,
        #[case] expected: &str,
        #[case] media_dir: Option<&str>
    ) {
        let presenter = if let Some(dir) = media_dir {
            HtmlPresenter::with_media_dir(dir)
        } else {
            HtmlPresenter::new()
        };

        let processed = presenter.process_content(input);
        assert_eq!(processed, expected);
    }
}