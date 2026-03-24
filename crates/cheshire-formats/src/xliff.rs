use crate::error::Result;
use crate::{FileFormat, ImportResult, SourceUnit};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::path::Path;

// --- Internal state machine for parsing ---

#[derive(Debug, PartialEq)]
enum State {
    Root,
    InFile,
    // XLIFF 1.2
    InTransUnit,
    InSource,
    InTarget,
    InNote,
    // XLIFF 2.0
    InUnit,
    InSegment,
    InSource20,
    InTarget20,
    InNote20,
}

struct XliffParser {
    version: u8, // 1 = 1.2, 2 = 2.0
    source_lang: Option<String>,
    target_lang: Option<String>,
    units: Vec<SourceUnit>,
    state: State,
    current_id: String,
    current_source: String,
    current_target: Option<String>,
    current_note: Option<String>,
    depth: usize,   // nesting depth inside source/target for inline elements
    unit_index: usize,
}

impl XliffParser {
    fn new() -> Self {
        Self {
            version: 1,
            source_lang: None,
            target_lang: None,
            units: Vec::new(),
            state: State::Root,
            current_id: String::new(),
            current_source: String::new(),
            current_target: None,
            current_note: None,
            depth: 0,
            unit_index: 0,
        }
    }

    fn push_unit(&mut self) {
        let source = self.current_source.trim().to_string();
        if !source.is_empty() {
            self.units.push(SourceUnit {
                id: if self.current_id.is_empty() {
                    self.unit_index.to_string()
                } else {
                    self.current_id.clone()
                },
                source,
                target: self.current_target.as_ref().map(|t| t.trim().to_string()).filter(|t| !t.is_empty()),
                note: self.current_note.as_ref().map(|n| n.trim().to_string()).filter(|n| !n.is_empty()),
            });
            self.unit_index += 1;
        }
        self.current_id.clear();
        self.current_source.clear();
        self.current_target = None;
        self.current_note = None;
        self.depth = 0;
    }

    fn attr(event: &quick_xml::events::BytesStart<'_>, name: &[u8]) -> Option<String> {
        event.attributes().flatten().find(|a| a.key.local_name().as_ref() == name).and_then(|a| {
            std::str::from_utf8(&a.value).ok().map(|s| s.to_string())
        })
    }
}

pub fn import(path: &Path) -> Result<ImportResult> {
    let content = std::fs::read_to_string(path)?;
    import_str(&content)
}

pub fn import_str(content: &str) -> Result<ImportResult> {
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(false);

    let mut parser = XliffParser::new();
    let mut buf = Vec::new();
    let mut detected_version = FileFormat::Xliff12;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local = e.local_name();
                let local = local.as_ref();
                match local {
                    b"xliff" => {
                        if let Some(v) = XliffParser::attr(e, b"version") {
                            if v.starts_with('2') {
                                parser.version = 2;
                                detected_version = FileFormat::Xliff20;
                            }
                        }
                        if parser.version == 2 {
                            if let Some(sl) = XliffParser::attr(e, b"srcLang") {
                                parser.source_lang = Some(sl);
                            }
                            if let Some(tl) = XliffParser::attr(e, b"trgLang") {
                                parser.target_lang = Some(tl);
                            }
                        }
                    }
                    b"file" => {
                        parser.state = State::InFile;
                        if parser.version == 1 {
                            if let Some(sl) = XliffParser::attr(e, b"source-language") {
                                parser.source_lang = Some(sl);
                            }
                            if let Some(tl) = XliffParser::attr(e, b"target-language") {
                                parser.target_lang = Some(tl);
                            }
                        }
                    }
                    b"trans-unit" if parser.version == 1 => {
                        parser.state = State::InTransUnit;
                        parser.current_id = XliffParser::attr(e, b"id").unwrap_or_default();
                    }
                    b"source" if parser.version == 1 && parser.state == State::InTransUnit => {
                        parser.state = State::InSource;
                        parser.depth = 0;
                    }
                    b"target" if parser.version == 1 && parser.state == State::InTransUnit => {
                        parser.state = State::InTarget;
                        parser.current_target = Some(String::new());
                        parser.depth = 0;
                    }
                    b"note" if parser.version == 1 && parser.state == State::InTransUnit => {
                        parser.state = State::InNote;
                        parser.current_note = Some(String::new());
                    }
                    b"unit" if parser.version == 2 => {
                        parser.state = State::InUnit;
                        parser.current_id = XliffParser::attr(e, b"id").unwrap_or_default();
                    }
                    b"segment" if parser.version == 2 && parser.state == State::InUnit => {
                        parser.state = State::InSegment;
                    }
                    b"source" if parser.version == 2 && parser.state == State::InSegment => {
                        parser.state = State::InSource20;
                        parser.depth = 0;
                    }
                    b"target" if parser.version == 2 && parser.state == State::InSegment => {
                        parser.state = State::InTarget20;
                        parser.current_target = Some(String::new());
                        parser.depth = 0;
                    }
                    b"note" if parser.version == 2 && (parser.state == State::InUnit || parser.state == State::InSegment) => {
                        parser.state = State::InNote20;
                        parser.current_note = Some(String::new());
                    }
                    _ => {
                        // Inline elements inside source/target: increment depth but keep state
                        match parser.state {
                            State::InSource | State::InSource20 |
                            State::InTarget | State::InTarget20 => {
                                parser.depth += 1;
                            }
                            _ => {}
                        }
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                // Self-closing inline tags like <x id="1"/>
                // Already in source/target — no text to add, nothing changes
                let _ = e;
            }
            Ok(Event::Text(ref t)) => {
                let text = t.unescape().unwrap_or_default();
                match parser.state {
                    State::InSource | State::InSource20 => {
                        parser.current_source.push_str(&text);
                    }
                    State::InTarget | State::InTarget20 => {
                        if let Some(ref mut tgt) = parser.current_target {
                            tgt.push_str(&text);
                        }
                    }
                    State::InNote | State::InNote20 => {
                        if let Some(ref mut note) = parser.current_note {
                            note.push_str(&text);
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                let local = e.local_name();
                let local = local.as_ref();
                match local {
                    b"trans-unit" if parser.version == 1 => {
                        parser.push_unit();
                        parser.state = State::InFile;
                    }
                    b"source" if parser.state == State::InSource => {
                        parser.state = State::InTransUnit;
                    }
                    b"target" if parser.state == State::InTarget => {
                        parser.state = State::InTransUnit;
                    }
                    b"note" if parser.state == State::InNote => {
                        parser.state = State::InTransUnit;
                    }
                    b"unit" if parser.version == 2 => {
                        parser.push_unit();
                        parser.state = State::InFile;
                    }
                    b"segment" if parser.state == State::InSegment => {
                        parser.state = State::InUnit;
                    }
                    b"source" if parser.state == State::InSource20 => {
                        parser.state = State::InSegment;
                    }
                    b"target" if parser.state == State::InTarget20 => {
                        parser.state = State::InSegment;
                    }
                    b"note" if parser.state == State::InNote20 => {
                        // restore to whichever we were in
                        parser.state = State::InUnit;
                    }
                    _ => {
                        // closing inline element
                        match parser.state {
                            State::InSource | State::InSource20 |
                            State::InTarget | State::InTarget20 => {
                                if parser.depth > 0 { parser.depth -= 1; }
                            }
                            _ => {}
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(e.into()),
            _ => {}
        }
        buf.clear();
    }

    Ok(ImportResult {
        units: parser.units,
        source_lang: parser.source_lang,
        target_lang: parser.target_lang,
        format: detected_version,
    })
}

/// Export units as a XLIFF 1.2 bilingual file.
pub fn export(path: &Path, units: &[SourceUnit], source_lang: &str, target_lang: &str) -> Result<()> {
    use std::io::Write;
    let mut out = std::fs::File::create(path)?;
    writeln!(out, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    writeln!(out, r#"<xliff version="1.2" xmlns="urn:oasis:names:tc:xliff:document:1.2">"#)?;
    writeln!(out, r#"  <file source-language="{source_lang}" target-language="{target_lang}" datatype="plaintext" original="document">"#)?;
    writeln!(out, "    <body>")?;
    for unit in units {
        let src = xml_escape(&unit.source);
        writeln!(out, r#"      <trans-unit id="{}">"#, xml_escape(&unit.id))?;
        writeln!(out, "        <source>{src}</source>")?;
        if let Some(ref tgt) = unit.target {
            writeln!(out, "        <target>{}</target>", xml_escape(tgt))?;
        } else {
            writeln!(out, "        <target/>")?;
        }
        if let Some(ref note) = unit.note {
            writeln!(out, "        <note>{}</note>", xml_escape(note))?;
        }
        writeln!(out, "      </trans-unit>")?;
    }
    writeln!(out, "    </body>")?;
    writeln!(out, "  </file>")?;
    writeln!(out, "</xliff>")?;
    Ok(())
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_tmp(content: &str, ext: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::Builder::new().suffix(ext).tempfile().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f
    }

    const XLIFF12_BASIC: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<xliff version="1.2" xmlns="urn:oasis:names:tc:xliff:document:1.2">
  <file source-language="en" target-language="ja" original="doc.txt" datatype="plaintext">
    <body>
      <trans-unit id="1">
        <source>Hello world.</source>
        <target>こんにちは世界。</target>
      </trans-unit>
      <trans-unit id="2">
        <source>Please sign the document.</source>
        <target>書類に署名してください。</target>
        <note>Legal context</note>
      </trans-unit>
    </body>
  </file>
</xliff>"#;

    const XLIFF12_INLINE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<xliff version="1.2" xmlns="urn:oasis:names:tc:xliff:document:1.2">
  <file source-language="en" target-language="de" original="ui.html" datatype="html">
    <body>
      <trans-unit id="btn1">
        <source>Click <g id="1">here</g> to continue.</source>
        <target>Klicken Sie <g id="1">hier</g>, um fortzufahren.</target>
      </trans-unit>
    </body>
  </file>
</xliff>"#;

    const XLIFF20_BASIC: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<xliff version="2.0" xmlns="urn:oasis:names:tc:xliff:document:2.0" srcLang="en" trgLang="fr">
  <file id="f1">
    <unit id="u1">
      <segment>
        <source>Good morning.</source>
        <target>Bonjour.</target>
      </segment>
    </unit>
    <unit id="u2">
      <segment>
        <source>Thank you very much.</source>
        <target>Merci beaucoup.</target>
      </segment>
    </unit>
  </file>
</xliff>"#;

    const XLIFF12_NO_TARGET: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<xliff version="1.2" xmlns="urn:oasis:names:tc:xliff:document:1.2">
  <file source-language="en" target-language="ja">
    <body>
      <trans-unit id="1">
        <source>Untranslated segment.</source>
      </trans-unit>
    </body>
  </file>
</xliff>"#;

    // ── XLIFF 1.2 ─────────────────────────────────────────────────────────────

    #[test]
    fn xliff12_parses_two_units() {
        let result = import_str(XLIFF12_BASIC).unwrap();
        assert_eq!(result.units.len(), 2);
        assert_eq!(result.format, FileFormat::Xliff12);
    }

    #[test]
    fn xliff12_source_lang_extracted() {
        let result = import_str(XLIFF12_BASIC).unwrap();
        assert_eq!(result.source_lang.as_deref(), Some("en"));
    }

    #[test]
    fn xliff12_target_lang_extracted() {
        let result = import_str(XLIFF12_BASIC).unwrap();
        assert_eq!(result.target_lang.as_deref(), Some("ja"));
    }

    #[test]
    fn xliff12_first_unit_source() {
        let result = import_str(XLIFF12_BASIC).unwrap();
        assert_eq!(result.units[0].source, "Hello world.");
    }

    #[test]
    fn xliff12_first_unit_target() {
        let result = import_str(XLIFF12_BASIC).unwrap();
        assert_eq!(result.units[0].target.as_deref(), Some("こんにちは世界。"));
    }

    #[test]
    fn xliff12_note_extracted() {
        let result = import_str(XLIFF12_BASIC).unwrap();
        assert_eq!(result.units[1].note.as_deref(), Some("Legal context"));
    }

    #[test]
    fn xliff12_ids_preserved() {
        let result = import_str(XLIFF12_BASIC).unwrap();
        assert_eq!(result.units[0].id, "1");
        assert_eq!(result.units[1].id, "2");
    }

    #[test]
    fn xliff12_inline_tags_text_preserved() {
        let result = import_str(XLIFF12_INLINE).unwrap();
        assert_eq!(result.units.len(), 1);
        assert!(result.units[0].source.contains("here"), "source should contain inline text");
        assert!(result.units[0].source.contains("Click"));
        assert!(result.units[0].source.contains("continue"));
    }

    #[test]
    fn xliff12_no_target_is_none() {
        let result = import_str(XLIFF12_NO_TARGET).unwrap();
        assert_eq!(result.units.len(), 1);
        assert!(result.units[0].target.is_none());
    }

    #[test]
    fn xliff12_empty_source_units_skipped() {
        let content = r#"<?xml version="1.0"?>
<xliff version="1.2" xmlns="urn:oasis:names:tc:xliff:document:1.2">
  <file source-language="en" target-language="de">
    <body>
      <trans-unit id="1"><source>   </source></trans-unit>
      <trans-unit id="2"><source>Real content.</source></trans-unit>
    </body>
  </file>
</xliff>"#;
        let result = import_str(content).unwrap();
        assert_eq!(result.units.len(), 1);
        assert_eq!(result.units[0].source, "Real content.");
    }

    // ── XLIFF 2.0 ─────────────────────────────────────────────────────────────

    #[test]
    fn xliff20_detects_version() {
        let result = import_str(XLIFF20_BASIC).unwrap();
        assert_eq!(result.format, FileFormat::Xliff20);
    }

    #[test]
    fn xliff20_lang_pair() {
        let result = import_str(XLIFF20_BASIC).unwrap();
        assert_eq!(result.source_lang.as_deref(), Some("en"));
        assert_eq!(result.target_lang.as_deref(), Some("fr"));
    }

    #[test]
    fn xliff20_two_units() {
        let result = import_str(XLIFF20_BASIC).unwrap();
        assert_eq!(result.units.len(), 2);
    }

    #[test]
    fn xliff20_unit_source_and_target() {
        let result = import_str(XLIFF20_BASIC).unwrap();
        assert_eq!(result.units[0].source, "Good morning.");
        assert_eq!(result.units[0].target.as_deref(), Some("Bonjour."));
    }

    #[test]
    fn xliff20_ids_preserved() {
        let result = import_str(XLIFF20_BASIC).unwrap();
        assert_eq!(result.units[0].id, "u1");
        assert_eq!(result.units[1].id, "u2");
    }

    // ── Export ────────────────────────────────────────────────────────────────

    #[test]
    fn export_roundtrip_source_and_target() {
        let units = vec![
            SourceUnit { id: "1".into(), source: "Hello.".into(), target: Some("Hallo.".into()), note: None },
            SourceUnit { id: "2".into(), source: "Goodbye.".into(), target: None, note: None },
        ];
        let tmp = tempfile::Builder::new().suffix(".xliff").tempfile().unwrap();
        export(tmp.path(), &units, "en", "de").unwrap();
        let reimported = import(tmp.path()).unwrap();
        assert_eq!(reimported.units.len(), 2);
        assert_eq!(reimported.units[0].source, "Hello.");
        assert_eq!(reimported.units[0].target.as_deref(), Some("Hallo."));
        assert!(reimported.units[1].target.is_none() || reimported.units[1].target.as_deref() == Some(""));
    }

    #[test]
    fn export_escapes_xml_special_chars() {
        let units = vec![
            SourceUnit { id: "1".into(), source: "Use <br> & \"quotes\".".into(), target: None, note: None },
        ];
        let tmp = tempfile::Builder::new().suffix(".xliff").tempfile().unwrap();
        export(tmp.path(), &units, "en", "ja").unwrap();
        let content = std::fs::read_to_string(tmp.path()).unwrap();
        assert!(content.contains("&lt;br&gt;"));
        assert!(content.contains("&amp;"));
    }

    #[test]
    fn export_sets_lang_pair() {
        let tmp = tempfile::Builder::new().suffix(".xliff").tempfile().unwrap();
        export(tmp.path(), &[], "en", "ja").unwrap();
        let reimported = import(tmp.path()).unwrap();
        assert_eq!(reimported.source_lang.as_deref(), Some("en"));
        assert_eq!(reimported.target_lang.as_deref(), Some("ja"));
    }

    #[test]
    fn export_includes_notes() {
        let units = vec![
            SourceUnit { id: "1".into(), source: "Sign here.".into(), target: None, note: Some("Legal term".into()) },
        ];
        let tmp = tempfile::Builder::new().suffix(".xliff").tempfile().unwrap();
        export(tmp.path(), &units, "en", "ja").unwrap();
        let content = std::fs::read_to_string(tmp.path()).unwrap();
        assert!(content.contains("Legal term"));
    }

    #[test]
    fn import_from_file_xliff12() {
        let f = write_tmp(XLIFF12_BASIC, ".xliff");
        let result = import(f.path()).unwrap();
        assert_eq!(result.units.len(), 2);
    }
}
