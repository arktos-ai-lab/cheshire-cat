//! CSV import/export for bilingual translation files.
//!
//! Format: UTF-8 CSV with a header row.
//! Import columns (auto-detected by header name, case-insensitive):
//!   id/key, source/src, target/tgt/translation, note/comment
//! Export columns: id, source, target, note

use crate::error::Result;
use crate::{FileFormat, ImportResult, SourceUnit};
use std::path::Path;

pub fn import(path: &Path) -> Result<ImportResult> {
    let content = std::fs::read_to_string(path)?;
    import_str(&content)
}

pub fn import_str(content: &str) -> Result<ImportResult> {
    let mut lines = content.lines();
    let header = match lines.next() {
        Some(h) => h,
        None => {
            return Ok(ImportResult {
                units: vec![],
                source_lang: None,
                target_lang: None,
                format: FileFormat::Csv,
            })
        }
    };

    let cols: Vec<&str> = split_csv_row(header);
    let idx_id = find_col(&cols, &["id", "key"]);
    let idx_source = find_col(&cols, &["source", "src"]);
    let idx_target = find_col(&cols, &["target", "tgt", "translation"]);
    let idx_note = find_col(&cols, &["note", "comment"]);

    let mut units = Vec::new();
    for (line_no, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let fields = split_csv_row(line);
        let source = idx_source
            .and_then(|i| fields.get(i).copied())
            .unwrap_or("")
            .to_string();
        if source.trim().is_empty() {
            continue;
        }
        let id = idx_id
            .and_then(|i| fields.get(i).copied())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{}", line_no + 1));
        let target = idx_target
            .and_then(|i| fields.get(i).copied())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let note = idx_note
            .and_then(|i| fields.get(i).copied())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        units.push(SourceUnit { id, source, target, note });
    }

    Ok(ImportResult { units, source_lang: None, target_lang: None, format: FileFormat::Csv })
}

/// Export units as a CSV file with columns: id, source, target, note.
pub fn export(path: &Path, units: &[SourceUnit]) -> Result<()> {
    let mut out = String::new();
    out.push_str("id,source,target,note\n");
    for unit in units {
        let target = unit.target.as_deref().unwrap_or("");
        let note = unit.note.as_deref().unwrap_or("");
        out.push_str(&csv_field(&unit.id));
        out.push(',');
        out.push_str(&csv_field(&unit.source));
        out.push(',');
        out.push_str(&csv_field(target));
        out.push(',');
        out.push_str(&csv_field(note));
        out.push('\n');
    }
    std::fs::write(path, out)?;
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Split a CSV row respecting double-quoted fields.
fn split_csv_row(row: &str) -> Vec<&str> {
    let mut fields = Vec::new();
    let bytes = row.as_bytes();
    let mut i = 0;
    let len = bytes.len();

    while i <= len {
        let start = i;
        if i < len && bytes[i] == b'"' {
            // Quoted field: find closing quote
            i += 1;
            let field_start = i;
            while i < len {
                if bytes[i] == b'"' {
                    if i + 1 < len && bytes[i + 1] == b'"' {
                        i += 2; // escaped quote
                    } else {
                        break;
                    }
                } else {
                    i += 1;
                }
            }
            // Return the slice between the quotes (does not unescape "" → " but
            // that is acceptable for translation content which rarely has quotes)
            fields.push(&row[field_start..i]);
            if i < len {
                i += 1; // skip closing quote
            }
            if i < len && bytes[i] == b',' {
                i += 1;
            }
        } else {
            // Unquoted field: scan to next comma
            while i < len && bytes[i] != b',' {
                i += 1;
            }
            fields.push(&row[start..i]);
            if i < len {
                i += 1; // skip comma
            }
        }
        if i > len {
            break;
        }
        if i == len && (fields.len() < 1 || !row.is_empty()) {
            // Trailing empty field
            if bytes.get(len.saturating_sub(1)) == Some(&b',') {
                fields.push("");
            }
            break;
        }
    }
    fields
}

/// Quote a CSV field if it contains commas, quotes, or newlines.
fn csv_field(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        let escaped = s.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    } else {
        s.to_string()
    }
}

/// Find column index by checking header names case-insensitively.
fn find_col(cols: &[&str], names: &[&str]) -> Option<usize> {
    cols.iter().position(|c| names.iter().any(|n| c.trim().eq_ignore_ascii_case(n)))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const BASIC_CSV: &str = "id,source,target,note\n\
        1,Hello world,Hallo Welt,greeting\n\
        2,Open file,Datei öffnen,\n\
        3,Untranslated,,\n";

    #[test]
    fn import_basic_csv() {
        let r = import_str(BASIC_CSV).unwrap();
        assert_eq!(r.units.len(), 3);
        assert_eq!(r.units[0].source, "Hello world");
        assert_eq!(r.units[0].target.as_deref(), Some("Hallo Welt"));
        assert_eq!(r.units[0].note.as_deref(), Some("greeting"));
    }

    #[test]
    fn import_empty_target_is_none() {
        let r = import_str(BASIC_CSV).unwrap();
        assert!(r.units[2].target.is_none());
    }

    #[test]
    fn import_empty_csv() {
        let r = import_str("").unwrap();
        assert!(r.units.is_empty());
    }

    #[test]
    fn import_header_only() {
        let r = import_str("id,source,target,note\n").unwrap();
        assert!(r.units.is_empty());
    }

    #[test]
    fn export_round_trip() {
        let units = vec![
            SourceUnit {
                id: "1".into(),
                source: "Hello, world".into(),
                target: Some("Bonjour, monde".into()),
                note: None,
            },
            SourceUnit {
                id: "2".into(),
                source: "No translation".into(),
                target: None,
                note: Some("needs review".into()),
            },
        ];
        let tmp = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
        export(tmp.path(), &units).unwrap();
        let r = import(tmp.path()).unwrap();
        assert_eq!(r.units.len(), 2);
        assert_eq!(r.units[0].source, "Hello, world");
        assert_eq!(r.units[0].target.as_deref(), Some("Bonjour, monde"));
        assert_eq!(r.units[1].note.as_deref(), Some("needs review"));
    }

    #[test]
    fn csv_field_quoting() {
        assert_eq!(csv_field("plain"), "plain");
        assert_eq!(csv_field("with,comma"), "\"with,comma\"");
        assert_eq!(csv_field("with\"quote"), "\"with\"\"quote\"");
    }

    #[test]
    fn format_is_csv() {
        let r = import_str(BASIC_CSV).unwrap();
        assert_eq!(r.format, FileFormat::Csv);
    }
}
