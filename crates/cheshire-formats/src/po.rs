use crate::error::Result;
use crate::{FileFormat, ImportResult, SourceUnit};
use std::path::Path;

pub fn import(path: &Path) -> Result<ImportResult> {
    let content = std::fs::read_to_string(path)?;
    Ok(import_str(&content))
}

/// Export units back to PO format (msgid + msgstr pairs).
pub fn export(path: &Path, units: &[SourceUnit]) -> Result<()> {
    use std::io::Write;
    let mut out = std::fs::File::create(path)?;
    for unit in units {
        if let Some(ref note) = unit.note {
            writeln!(out, "#. {note}")?;
        }
        write_po_string(&mut out, "msgid", &unit.source)?;
        if let Some(ref tgt) = unit.target {
            write_po_string(&mut out, "msgstr", tgt)?;
        } else {
            writeln!(out, "msgstr \"\"")?;
        }
        writeln!(out)?;
    }
    Ok(())
}

fn write_po_string(out: &mut dyn std::io::Write, keyword: &str, value: &str) -> std::io::Result<()> {
    if value.contains('\n') {
        writeln!(out, "{keyword} \"\"")?;
        for line in value.split('\n') {
            writeln!(out, "\"{}\\n\"", po_escape(line))?;
        }
    } else {
        writeln!(out, "{keyword} \"{}\"", po_escape(value))?;
    }
    Ok(())
}

fn po_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn po_unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some(other) => { out.push('\\'); out.push(other); }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Parse a quoted PO string token: `"content"` → `content` (unescaped).
/// Returns None if the line doesn't start with a quote.
fn parse_quoted(line: &str) -> Option<String> {
    let line = line.trim();
    if line.starts_with('"') && line.ends_with('"') && line.len() >= 2 {
        let inner = &line[1..line.len()-1];
        Some(po_unescape(inner))
    } else {
        None
    }
}

pub fn import_str(content: &str) -> ImportResult {
    let mut units: Vec<SourceUnit> = Vec::new();
    let mut current_msgid = String::new();
    let mut current_msgstr = String::new();
    let mut current_note: Option<String> = None;
    let mut is_fuzzy = false;
    let mut state = PoState::None;
    let mut unit_index = 0usize;

    #[derive(PartialEq)]
    enum PoState { None, InMsgid, InMsgstr }

    fn flush_unit(
        msgid: &str,
        msgstr: &str,
        note: &Option<String>,
        index: &mut usize,
        units: &mut Vec<SourceUnit>,
        fuzzy: bool,
    ) {
        let source = msgid.trim().to_string();
        if source.is_empty() {
            return; // header unit
        }
        units.push(SourceUnit {
            id: index.to_string(),
            source,
            target: if msgstr.is_empty() || fuzzy { None } else { Some(msgstr.trim().to_string()) },
            note: note.clone(),
        });
        *index += 1;
    }

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("#,") {
            is_fuzzy = trimmed.contains("fuzzy");
            continue;
        }
        if trimmed.starts_with("#.") {
            let note_text = trimmed[2..].trim().to_string();
            current_note = Some(note_text);
            continue;
        }
        if trimmed.starts_with('#') {
            continue; // other comments
        }

        if trimmed.is_empty() {
            if state != PoState::None || !current_msgid.is_empty() {
                flush_unit(&current_msgid, &current_msgstr, &current_note,
                           &mut unit_index, &mut units, is_fuzzy);
                current_msgid.clear();
                current_msgstr.clear();
                current_note = None;
                is_fuzzy = false;
                state = PoState::None;
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("msgid ") {
            if state != PoState::None {
                flush_unit(&current_msgid, &current_msgstr, &current_note,
                           &mut unit_index, &mut units, is_fuzzy);
                current_msgid.clear();
                current_msgstr.clear();
                current_note = None;
                is_fuzzy = false;
            }
            state = PoState::InMsgid;
            if let Some(s) = parse_quoted(rest) {
                current_msgid = s;
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("msgid_plural ") {
            // Use the plural form as source for now; skip msgstr[0..n]
            state = PoState::InMsgid;
            if let Some(s) = parse_quoted(rest) {
                if current_msgid.is_empty() {
                    current_msgid = s;
                }
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("msgstr ") {
            state = PoState::InMsgstr;
            if let Some(s) = parse_quoted(rest) {
                current_msgstr = s;
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("msgstr[0] ") {
            state = PoState::InMsgstr;
            if let Some(s) = parse_quoted(rest) {
                current_msgstr = s;
            }
            continue;
        }

        // Continuation strings
        if trimmed.starts_with('"') {
            if let Some(s) = parse_quoted(trimmed) {
                match state {
                    PoState::InMsgid => current_msgid.push_str(&s),
                    PoState::InMsgstr => current_msgstr.push_str(&s),
                    PoState::None => {}
                }
            }
        }
    }

    // Flush last unit
    if !current_msgid.is_empty() {
        flush_unit(&current_msgid, &current_msgstr, &current_note,
                   &mut unit_index, &mut units, is_fuzzy);
    }

    ImportResult {
        units,
        source_lang: None,
        target_lang: None,
        format: FileFormat::Po,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASIC_PO: &str = r#"
msgid "Hello world."
msgstr "Hallo Welt."

msgid "Please sign the document."
msgstr "Bitte unterschreiben Sie das Dokument."
"#;

    const PO_WITH_COMMENT: &str = r#"
#. Translator note: this is a greeting
msgid "Good morning."
msgstr "Guten Morgen."
"#;

    const PO_FUZZY: &str = r#"
#, fuzzy
msgid "Submit form."
msgstr "Formular abschicken."
"#;

    const PO_MULTILINE: &str = r#"
msgid ""
"First line.\n"
"Second line."
msgstr ""
"Erste Zeile.\n"
"Zweite Zeile."
"#;

    const PO_NO_TRANSLATION: &str = r#"
msgid "Untranslated string."
msgstr ""
"#;

    // ── Basic parsing ──────────────────────────────────────────────────────────

    #[test]
    fn basic_two_units() {
        let r = import_str(BASIC_PO);
        assert_eq!(r.units.len(), 2);
    }

    #[test]
    fn source_text_extracted() {
        let r = import_str(BASIC_PO);
        assert_eq!(r.units[0].source, "Hello world.");
    }

    #[test]
    fn target_text_extracted() {
        let r = import_str(BASIC_PO);
        assert_eq!(r.units[0].target.as_deref(), Some("Hallo Welt."));
    }

    #[test]
    fn extracted_comment_becomes_note() {
        let r = import_str(PO_WITH_COMMENT);
        assert_eq!(r.units[0].note.as_deref(), Some("Translator note: this is a greeting"));
    }

    #[test]
    fn fuzzy_translation_treated_as_untranslated() {
        let r = import_str(PO_FUZZY);
        assert_eq!(r.units.len(), 1);
        assert!(r.units[0].target.is_none(), "fuzzy entries should have None target");
    }

    #[test]
    fn multiline_msgid_joined() {
        let r = import_str(PO_MULTILINE);
        assert_eq!(r.units.len(), 1);
        assert_eq!(r.units[0].source, "First line.\nSecond line.");
    }

    #[test]
    fn multiline_msgstr_joined() {
        let r = import_str(PO_MULTILINE);
        assert_eq!(r.units[0].target.as_deref(), Some("Erste Zeile.\nZweite Zeile."));
    }

    #[test]
    fn empty_msgstr_is_none_target() {
        let r = import_str(PO_NO_TRANSLATION);
        assert_eq!(r.units.len(), 1);
        assert!(r.units[0].target.is_none());
    }

    #[test]
    fn header_unit_skipped() {
        // PO files start with an empty msgid header
        let content = r#"msgid ""
msgstr ""
"Content-Type: text/plain; charset=UTF-8\n"

msgid "Real string."
msgstr "Echte Zeichenkette."
"#;
        let r = import_str(content);
        assert_eq!(r.units.len(), 1);
        assert_eq!(r.units[0].source, "Real string.");
    }

    #[test]
    fn hash_comments_ignored() {
        let content = "# translator comment\n#: src/file.rs:42\nmsgid \"Hello.\"\nmsgstr \"Hallo.\"\n";
        let r = import_str(content);
        assert_eq!(r.units.len(), 1);
    }

    #[test]
    fn format_is_po() {
        let r = import_str(BASIC_PO);
        assert_eq!(r.format, FileFormat::Po);
    }

    // ── Export ────────────────────────────────────────────────────────────────

    #[test]
    fn po_export_roundtrip() {
        let units = vec![
            SourceUnit { id: "0".into(), source: "Hello.".into(), target: Some("Hallo.".into()), note: None },
            SourceUnit { id: "1".into(), source: "Goodbye.".into(), target: None, note: None },
        ];
        let tmp = tempfile::Builder::new().suffix(".po").tempfile().unwrap();
        export(tmp.path(), &units).unwrap();
        let reimported = import(tmp.path()).unwrap();
        assert_eq!(reimported.units.len(), 2);
        assert_eq!(reimported.units[0].source, "Hello.");
        assert_eq!(reimported.units[0].target.as_deref(), Some("Hallo."));
        assert!(reimported.units[1].target.is_none());
    }

    #[test]
    fn po_export_note_becomes_extracted_comment() {
        let units = vec![
            SourceUnit { id: "0".into(), source: "Click here.".into(), target: None, note: Some("UI button".into()) },
        ];
        let tmp = tempfile::Builder::new().suffix(".po").tempfile().unwrap();
        export(tmp.path(), &units).unwrap();
        let content = std::fs::read_to_string(tmp.path()).unwrap();
        assert!(content.contains("#. UI button"));
    }

    #[test]
    fn escape_unescape_roundtrip() {
        let original = r#"Say "hello" & goodbye\world."#;
        let escaped = po_escape(original);
        let unescaped = po_unescape(&escaped);
        assert_eq!(unescaped, original);
    }
}
