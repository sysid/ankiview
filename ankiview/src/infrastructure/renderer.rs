// src/infrastructure/renderer.rs
use anyhow::{Context, Result};
use regex::Regex;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::Builder;
use tracing::instrument;

#[derive(Debug)]
pub struct ContentRenderer {
    latex_regex: Regex,
    // Keep last temp dir alive to prevent deletion
    _temp_dir: Option<Arc<tempfile::TempDir>>,
}

impl ContentRenderer {
    pub fn new() -> Self {
        Self {
            latex_regex: Regex::new(r"```(?:tex|latex)?\n(\$\$[\s\S]*?\$\$)\n```").unwrap(),
            _temp_dir: None,
        }
    }

    #[instrument(level = "trace")]
    pub fn process_latex(&self, content: &str) -> String {
        self.latex_regex.replace_all(content, "$1").into_owned()
    }

    pub fn create_temp_file(&mut self, content: &str) -> Result<PathBuf> {
        let temp_dir = Builder::new()
            .prefix("anki-viewer-")
            .rand_bytes(5)
            .tempdir()
            .context("Failed to create temporary directory")?;

        let file_path = temp_dir.path().join("note.html");

        File::create(&file_path)
            .with_context(|| format!("Failed to create temp file at {}", file_path.display()))?
            .write_all(content.as_bytes())
            .context("Failed to write content to temporary file")?;

        // Store temp_dir to keep it alive
        self._temp_dir = Some(Arc::new(temp_dir));

        Ok(file_path)
    }

    // Change the method signature to &mut self since we need to modify _temp_dir
    #[instrument(level = "debug")]
    pub fn open_in_browser(&mut self, path: &PathBuf) -> Result<()> {
        let path_str = path.to_str().context("Failed to convert path to string")?;

        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(path_str)
                .spawn()
                .context("Failed to open browser")?;
        }
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(["/C", "start", path_str])
                .spawn()
                .context("Failed to open browser")?;
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(path_str)
                .spawn()
                .context("Failed to open browser")?;
        }

        // Keep the temp directory alive briefly
        std::thread::sleep(std::time::Duration::from_millis(200));

        Ok(())
    }
}

impl Default for ContentRenderer {
    fn default() -> Self {
        Self::new()
    }
}
