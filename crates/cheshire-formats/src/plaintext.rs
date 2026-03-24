use crate::error::Result;
use crate::{FileFormat, ImportResult, SourceUnit};
use std::path::Path;

/// Import a plain text file. Each non-empty paragraph (separated by one or
/// more blank lines) becomes one `SourceUnit`.
pub fn import(path: &Path) -> Result<ImportResult> {
    let content = std::fs::read_to_string(path)?;
    Ok(import_str(&content))
}

pub fn import_str(content: &str) -> ImportResult {
    let mut units: Vec<SourceUnit> = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    let mut index = 0usize;

    for line in content.lines() {
        if line.trim().is_empty() {
            flush(&mut current, &mut units, &mut index);
        } else {
            current.push(line);
        }
    }
    flush(&mut current, &mut units, &mut index);

    ImportResult {
        units,
        source_lang: None,
        target_lang: None,
        format: FileFormat::PlainText,
    }
}

fn flush(current: &mut Vec<&str>, units: &mut Vec<SourceUnit>, index: &mut usize) {
    if !current.is_empty() {
        let text = current.join("\n").trim().to_string();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_paragraph() {
        let r = import_str("Hello world.");
        assert_eq!(r.units.len(), 1);
        assert_eq!(r.units[0].source, "Hello world.");
    }

    #[test]
    fn two_paragraphs_split_by_blank_line() {
        let r = import_str("First paragraph.\n\nSecond paragraph.");
        assert_eq!(r.units.len(), 2);
        assert_eq!(r.units[0].source, "First paragraph.");
        assert_eq!(r.units[1].source, "Second paragraph.");
    }

    #[test]
    fn multiple_blank_lines_treated_as_one_separator() {
        let r = import_str("Para one.\n\n\n\nPara two.");
        assert_eq!(r.units.len(), 2);
    }

    #[test]
    fn multiline_paragraph_joined() {
        let r = import_str("Line one.\nLine two.\nLine three.\n\nNew paragraph.");
        assert_eq!(r.units.len(), 2);
        assert_eq!(r.units[0].source, "Line one.\nLine two.\nLine three.");
    }

    #[test]
    fn empty_input_returns_empty() {
        let r = import_str("");
        assert!(r.units.is_empty());
    }

    #[test]
    fn whitespace_only_returns_empty() {
        let r = import_str("   \n  \n   ");
        assert!(r.units.is_empty());
    }

    #[test]
    fn indices_sequential() {
        let r = import_str("A.\n\nB.\n\nC.");
        assert_eq!(r.units[0].id, "0");
        assert_eq!(r.units[1].id, "1");
        assert_eq!(r.units[2].id, "2");
    }

    #[test]
    fn targets_are_none() {
        let r = import_str("Any text.");
        assert!(r.units[0].target.is_none());
    }

    #[test]
    fn format_is_plain_text() {
        let r = import_str("Text.");
        assert_eq!(r.format, FileFormat::PlainText);
    }

    #[test]
    fn leading_trailing_blank_lines_ignored() {
        let r = import_str("\n\nFirst.\n\nSecond.\n\n");
        assert_eq!(r.units.len(), 2);
    }
}
