use regex::Regex;
use lazy_static::lazy_static;
use std::path::PathBuf;

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
}
