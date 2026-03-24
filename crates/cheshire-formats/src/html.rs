use crate::error::Result;
use crate::{FileFormat, ImportResult, SourceUnit};
use regex::Regex;
use std::path::Path;
use std::sync::OnceLock;

pub fn import(path: &Path) -> Result<ImportResult> {
    let content = std::fs::read_to_string(path)?;
    Ok(import_str(&content))
}

pub fn import_str(html: &str) -> ImportResult {
    let text = extract_text(html);
    // Reuse the plain text paragraph splitter logic
    let mut units = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    let mut index = 0usize;

    for line in text.lines() {
        if line.trim().is_empty() {
            flush_para(&mut current, &mut units, &mut index);
        } else {
            current.push(line);
        }
    }
    flush_para(&mut current, &mut units, &mut index);

    ImportResult {
        units,
        source_lang: None,
        target_lang: None,
        format: FileFormat::Html,
    }
}

fn flush_para(current: &mut Vec<&str>, units: &mut Vec<SourceUnit>, index: &mut usize) {
    if !current.is_empty() {
        let text = current.join(" ").trim().to_string();
        if !text.is_empty() {
            units.push(SourceUnit {
                id: index.to_string(),
                source: text,
                target: None,
                note: None,
            });
            *index += 1;
        }
        current.clear();
    }
}

/// Strip HTML tags and convert block elements to newlines.
/// Removes `<script>` and `<style>` blocks entirely.
fn extract_text(html: &str) -> String {
    static SCRIPT_RE: OnceLock<Regex> = OnceLock::new();
    static STYLE_RE: OnceLock<Regex> = OnceLock::new();
    static BLOCK_RE: OnceLock<Regex> = OnceLock::new();
    static TAG_RE: OnceLock<Regex> = OnceLock::new();
    static SPACE_RE: OnceLock<Regex> = OnceLock::new();

    let script_re = SCRIPT_RE.get_or_init(|| {
        Regex::new(r"(?si)<script[^>]*>.*?</script>").unwrap()
    });
    let style_re = STYLE_RE.get_or_init(|| {
        Regex::new(r"(?si)<style[^>]*>.*?</style>").unwrap()
    });
    let block_re = BLOCK_RE.get_or_init(|| {
        Regex::new(r"(?i)</?(?:p|div|h[1-6]|li|tr|br|blockquote|pre|article|section|header|footer|nav|main|aside)[^>]*>").unwrap()
    });
    let tag_re = TAG_RE.get_or_init(|| Regex::new(r"<[^>]+>").unwrap());
    let space_re = SPACE_RE.get_or_init(|| Regex::new(r"\n{3,}").unwrap());

    let s = script_re.replace_all(html, "");
    let s = style_re.replace_all(&s, "");
    let s = block_re.replace_all(&s, "\n");
    let s = tag_re.replace_all(&s, "");
    // Decode common HTML entities
    let s = s
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
        .replace("&#160;", " ");
    let s = space_re.replace_all(&s, "\n\n");
    s.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_paragraph_extracted() {
        let r = import_str("<p>Hello world.</p>");
        assert_eq!(r.units.len(), 1);
        assert_eq!(r.units[0].source, "Hello world.");
    }

    #[test]
    fn two_paragraphs_two_units() {
        let r = import_str("<p>First.</p><p>Second.</p>");
        assert_eq!(r.units.len(), 2);
    }

    #[test]
    fn script_content_excluded() {
        let r = import_str("<script>var x = 1;</script><p>Visible text.</p>");
        assert_eq!(r.units.len(), 1);
        assert_eq!(r.units[0].source, "Visible text.");
        assert!(!r.units.iter().any(|u| u.source.contains("var x")));
    }

    #[test]
    fn style_content_excluded() {
        let r = import_str("<style>body { color: red; }</style><p>Content.</p>");
        assert_eq!(r.units.len(), 1);
        assert!(!r.units.iter().any(|u| u.source.contains("color")));
    }

    #[test]
    fn inline_tags_stripped() {
        let r = import_str("<p>Click <strong>here</strong> to continue.</p>");
        assert_eq!(r.units.len(), 1);
        assert_eq!(r.units[0].source, "Click here to continue.");
    }

    #[test]
    fn headings_extracted() {
        let r = import_str("<h1>Chapter One</h1><p>Introduction text.</p>");
        assert_eq!(r.units.len(), 2);
        assert_eq!(r.units[0].source, "Chapter One");
    }

    #[test]
    fn html_entities_decoded() {
        let r = import_str("<p>Use &amp; for ampersand; &lt;tag&gt; for angle.</p>");
        assert_eq!(r.units[0].source, "Use & for ampersand; <tag> for angle.");
    }

    #[test]
    fn empty_html_returns_empty() {
        let r = import_str("");
        assert!(r.units.is_empty());
    }

    #[test]
    fn format_is_html() {
        let r = import_str("<p>Text.</p>");
        assert_eq!(r.format, FileFormat::Html);
    }

    #[test]
    fn full_html_document() {
        let html = r#"<!DOCTYPE html>
<html>
<head><title>Test</title><style>body{}</style></head>
<body>
  <h1>Document Title</h1>
  <p>First paragraph with <em>emphasis</em>.</p>
  <p>Second paragraph.</p>
</body>
</html>"#;
        let r = import_str(html);
        assert!(r.units.len() >= 3);
        assert!(r.units.iter().any(|u| u.source.contains("Document Title")));
        assert!(r.units.iter().any(|u| u.source.contains("First paragraph")));
        assert!(r.units.iter().any(|u| u.source.contains("emphasis")));
    }
}
