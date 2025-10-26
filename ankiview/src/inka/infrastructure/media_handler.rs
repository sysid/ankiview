use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    // Match markdown images: ![alt](path)
    static ref MD_IMAGE_REGEX: Regex = Regex::new(r"!\[.*?\]\(([^)]+)\)")
        .expect("Failed to compile markdown image regex");

    // Match HTML img tags: <img src="path">
    static ref HTML_IMAGE_REGEX: Regex = Regex::new(r#"<img[^>]+src="([^"]+)""#)
        .expect("Failed to compile HTML image regex");
}

/// Extract image paths from markdown content
/// Supports both markdown syntax ![alt](path) and HTML <img src="path">
pub fn extract_image_paths(markdown: &str) -> Vec<String> {
    let mut paths = Vec::new();

    // Extract markdown format images
    for cap in MD_IMAGE_REGEX.captures_iter(markdown) {
        if let Some(path_match) = cap.get(1) {
            let path = path_match.as_str();
            // Skip HTTP(S) URLs
            if !path.starts_with("http://") && !path.starts_with("https://") {
                paths.push(path.to_string());
            }
        }
    }

    // Extract HTML format images
    for cap in HTML_IMAGE_REGEX.captures_iter(markdown) {
        if let Some(path_match) = cap.get(1) {
            let path = path_match.as_str();
            // Skip HTTP(S) URLs
            if !path.starts_with("http://") && !path.starts_with("https://") {
                paths.push(path.to_string());
            }
        }
    }

    paths
}

/// Copy a media file to Anki's collection.media directory
/// Returns the filename (not full path) that Anki will use
pub fn copy_media_to_anki(
    source_path: &std::path::Path,
    media_dir: &std::path::Path,
) -> anyhow::Result<String> {
    use anyhow::Context;

    // Extract filename from source path
    let filename = source_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;

    let dest_path = media_dir.join(filename);

    // Skip copying if file already exists in media directory
    if !dest_path.exists() {
        std::fs::copy(source_path, &dest_path)
            .context("Failed to copy media file")?;
    }

    Ok(filename.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_markdown_image_when_extracting_then_returns_path() {
        let markdown = "Some text ![alt text](images/photo.png) more text";
        let paths = extract_image_paths(markdown);

        assert_eq!(paths, vec!["images/photo.png"]);
    }

    #[test]
    fn given_multiple_images_when_extracting_then_returns_all_paths() {
        let markdown = r#"
![First](image1.png)
Some text
![Second](path/to/image2.jpg)
More text
![Third](../relative/image3.gif)
"#;
        let paths = extract_image_paths(markdown);

        assert_eq!(paths.len(), 3);
        assert!(paths.contains(&"image1.png".to_string()));
        assert!(paths.contains(&"path/to/image2.jpg".to_string()));
        assert!(paths.contains(&"../relative/image3.gif".to_string()));
    }

    #[test]
    fn given_html_img_tag_when_extracting_then_returns_path() {
        let markdown = r#"Some text <img src="diagrams/flow.png"> more text"#;
        let paths = extract_image_paths(markdown);

        assert_eq!(paths, vec!["diagrams/flow.png"]);
    }

    #[test]
    fn given_mixed_formats_when_extracting_then_returns_all() {
        let markdown = r#"
Markdown: ![logo](logo.png)
HTML: <img src="banner.jpg">
Another: ![icon](icons/star.svg)
"#;
        let paths = extract_image_paths(markdown);

        assert_eq!(paths.len(), 3);
        assert!(paths.contains(&"logo.png".to_string()));
        assert!(paths.contains(&"banner.jpg".to_string()));
        assert!(paths.contains(&"icons/star.svg".to_string()));
    }

    #[test]
    fn given_no_images_when_extracting_then_returns_empty() {
        let markdown = "Just text with no images at all";
        let paths = extract_image_paths(markdown);

        assert!(paths.is_empty());
    }

    #[test]
    fn given_absolute_urls_when_extracting_then_excludes_them() {
        let markdown = r#"
Local: ![local](image.png)
HTTP: ![remote](http://example.com/image.jpg)
HTTPS: ![secure](https://example.com/photo.png)
"#;
        let paths = extract_image_paths(markdown);

        // Should only return local path, not HTTP(S) URLs
        assert_eq!(paths, vec!["image.png"]);
    }

    #[test]
    fn given_source_file_when_copying_then_file_appears_in_media_dir() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let source_file = temp_dir.path().join("test_image.png");
        fs::write(&source_file, b"fake image data").unwrap();

        let media_dir = temp_dir.path().join("collection.media");
        fs::create_dir(&media_dir).unwrap();

        let filename = copy_media_to_anki(&source_file, &media_dir).unwrap();

        // Should return just the filename
        assert_eq!(filename, "test_image.png");

        // File should exist in media directory
        let dest_path = media_dir.join(&filename);
        assert!(dest_path.exists());

        // Content should match
        let content = fs::read(&dest_path).unwrap();
        assert_eq!(content, b"fake image data");
    }

    #[test]
    fn given_existing_file_when_copying_then_skips_duplicate() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let source_file = temp_dir.path().join("image.png");
        fs::write(&source_file, b"original").unwrap();

        let media_dir = temp_dir.path().join("collection.media");
        fs::create_dir(&media_dir).unwrap();

        // Pre-create the file in media dir
        let existing_file = media_dir.join("image.png");
        fs::write(&existing_file, b"already exists").unwrap();

        // Copy should succeed and return filename
        let filename = copy_media_to_anki(&source_file, &media_dir).unwrap();
        assert_eq!(filename, "image.png");

        // Should not overwrite existing file
        let content = fs::read(&existing_file).unwrap();
        assert_eq!(content, b"already exists");
    }

    #[test]
    fn given_nonexistent_source_when_copying_then_returns_error() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let nonexistent = temp_dir.path().join("doesnt_exist.png");

        let media_dir = temp_dir.path().join("collection.media");
        fs::create_dir(&media_dir).unwrap();

        let result = copy_media_to_anki(&nonexistent, &media_dir);
        assert!(result.is_err());
    }

    #[test]
    fn given_file_with_path_when_copying_then_returns_basename() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("images");
        fs::create_dir(&subdir).unwrap();

        let source_file = subdir.join("photo.jpg");
        fs::write(&source_file, b"photo data").unwrap();

        let media_dir = temp_dir.path().join("collection.media");
        fs::create_dir(&media_dir).unwrap();

        let filename = copy_media_to_anki(&source_file, &media_dir).unwrap();

        // Should return just filename, not path
        assert_eq!(filename, "photo.jpg");

        // File should be in media dir root (not in subdirectory)
        assert!(media_dir.join("photo.jpg").exists());
    }
}
