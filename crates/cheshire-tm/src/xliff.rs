//! XLIFF 1.2 / 2.0 import and export for the translation memory.
//!
//! Import extracts confirmed source+target pairs from bilingual XLIFF files
//! and inserts them as TM records.  Export writes TM records as XLIFF 1.2.

use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::{fs::File, str};

use chrono::Utc;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};

use crate::{
    error::Result,
    record::{ImportStats, Record},
    store::TmStore,
};

// ── Public entry points ───────────────────────────────────────────────────────

/// Import translation units from an XLIFF 1.2 or 2.0 bilingual file into
/// the store.  Only units that have both a source and a non-empty target are
/// imported; untranslated segments are silently skipped.
pub async fn import(path: &Path, store: &TmStore) -> Result<ImportStats> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let records = parse_xliff(reader)?;

    let mut stats = ImportStats::default();
    for record in records {
        match store.insert(&record).await {
            Ok(_) => stats.imported += 1,
            Err(crate::Error::Database(e))
                if e.to_string().contains("UNIQUE constraint failed") =>
            {
                stats.skipped_duplicates += 1;
            }
            Err(_) => stats.skipped_errors += 1,
        }
    }
    Ok(stats)
}

/// Export TM records to an XLIFF 1.2 file.
pub async fn export(path: &Path, records: &[Record]) -> Result<()> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    write_xliff(writer, records)
}

// ── Parser ────────────────────────────────────────────────────────────────────

fn parse_xliff(reader: impl BufRead) -> Result<Vec<Record>> {
    let mut xml = Reader::from_reader(reader);
    xml.config_mut().trim_text(true);

    let mut records: Vec<Record> = Vec::new();
    let mut buf = Vec::new();

    // Detected language pair from the <file> element.
    let mut file_source_lang = String::new();
    let mut file_target_lang = String::new();

    // Per-unit state
    let mut unit_id = String::new();
    let mut unit_source = String::new();
    let mut unit_target = String::new();
    let mut unit_source_lang = String::new();
    let mut unit_target_lang = String::new();

    // Text-capture context
    #[derive(PartialEq)]
    enum Capture { None, Source, Target }
    let mut capture = Capture::None;
    let mut depth: u32 = 0; // nesting depth inside the capture element

    loop {
        match xml.read_event_into(&mut buf)? {
            // ── <xliff> root — XLIFF 2.0 puts srcLang/trgLang here ────────
            Event::Start(ref e) | Event::Empty(ref e) if e.name().local_name().as_ref() == b"xliff" => {
                for attr in e.attributes().flatten() {
                    match attr.key.as_ref() {
                        b"srcLang" => {
                            file_source_lang =
                                String::from_utf8_lossy(&attr.value).to_lowercase();
                        }
                        b"trgLang" => {
                            file_target_lang =
                                String::from_utf8_lossy(&attr.value).to_lowercase();
                        }
                        _ => {}
                    }
                }
            }

            // ── <file> — XLIFF 1.2 puts source-language/target-language here
            Event::Start(ref e) | Event::Empty(ref e) if e.name().local_name().as_ref() == b"file" => {
                for attr in e.attributes().flatten() {
                    match attr.key.as_ref() {
                        b"source-language" => {
                            file_source_lang =
                                String::from_utf8_lossy(&attr.value).to_lowercase();
                        }
                        b"target-language" => {
                            file_target_lang =
                                String::from_utf8_lossy(&attr.value).to_lowercase();
                        }
                        _ => {}
                    }
                }
            }

            // ── XLIFF 1.2: <trans-unit id="…"> ────────────────────────────
            Event::Start(ref e) if e.name().as_ref() == b"trans-unit" => {
                unit_id.clear();
                unit_source.clear();
                unit_target.clear();
                unit_source_lang = file_source_lang.clone();
                unit_target_lang = file_target_lang.clone();
                for attr in e.attributes().flatten() {
                    if attr.key.as_ref() == b"id" {
                        unit_id = String::from_utf8_lossy(&attr.value).into_owned();
                    }
                }
            }

            // ── XLIFF 2.0: <unit id="…"> ──────────────────────────────────
            Event::Start(ref e) if e.name().as_ref() == b"unit" => {
                unit_id.clear();
                unit_source.clear();
                unit_target.clear();
                unit_source_lang = file_source_lang.clone();
                unit_target_lang = file_target_lang.clone();
                for attr in e.attributes().flatten() {
                    if attr.key.as_ref() == b"id" {
                        unit_id = String::from_utf8_lossy(&attr.value).into_owned();
                    }
                }
            }

            // ── <source> ──────────────────────────────────────────────────
            Event::Start(ref e) if e.name().as_ref() == b"source" => {
                capture = Capture::Source;
                depth = 0;
                unit_source.clear();
                for attr in e.attributes().flatten() {
                    if attr.key.as_ref() == b"xml:lang" {
                        unit_source_lang =
                            String::from_utf8_lossy(&attr.value).to_lowercase();
                    }
                }
            }

            // ── <target> ──────────────────────────────────────────────────
            Event::Start(ref e) if e.name().as_ref() == b"target" => {
                capture = Capture::Target;
                depth = 0;
                unit_target.clear();
                for attr in e.attributes().flatten() {
                    if attr.key.as_ref() == b"xml:lang" {
                        unit_target_lang =
                            String::from_utf8_lossy(&attr.value).to_lowercase();
                    }
                }
            }

            // ── Nested elements inside source/target — skip tags, keep text
            Event::Start(_) if capture != Capture::None => {
                depth += 1;
            }
            Event::Empty(_) if capture != Capture::None => {}

            Event::End(ref e) => match e.name().as_ref() {
                b"source" | b"target" if depth == 0 => {
                    capture = Capture::None;
                }
                b"source" | b"target" => {
                    depth -= 1;
                }
                b"trans-unit" | b"unit" => {
                    // Commit the unit if both sides are non-empty
                    let src = unit_source.trim().to_string();
                    let tgt = unit_target.trim().to_string();
                    if !src.is_empty() && !tgt.is_empty() {
                        let sl = if unit_source_lang.is_empty() {
                            file_source_lang.clone()
                        } else {
                            unit_source_lang.clone()
                        };
                        let tl = if unit_target_lang.is_empty() {
                            file_target_lang.clone()
                        } else {
                            unit_target_lang.clone()
                        };
                        records.push(Record::new(src, tgt, sl, tl));
                    }
                }
                _ => {}
            },

            // ── Text content ──────────────────────────────────────────────
            Event::Text(ref e) => {
                let text = e.unescape().unwrap_or_default();
                match capture {
                    Capture::Source => unit_source.push_str(&text),
                    Capture::Target => unit_target.push_str(&text),
                    Capture::None => {}
                }
            }

            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(records)
}

// ── Writer ────────────────────────────────────────────────────────────────────

fn write_xliff(writer: impl Write, records: &[Record]) -> Result<()> {
    let mut xml = Writer::new_with_indent(writer, b' ', 2);

    xml.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

    // Derive language pair from records (use first record, or empty strings)
    let (src_lang, tgt_lang) = records
        .first()
        .map(|r| (r.source_lang.as_str(), r.target_lang.as_str()))
        .unwrap_or(("", ""));

    // <xliff>
    let mut xliff_el = BytesStart::new("xliff");
    xliff_el.push_attribute(("version", "1.2"));
    xliff_el.push_attribute(("xmlns", "urn:oasis:names:tc:xliff:document:1.2"));
    xml.write_event(Event::Start(xliff_el))?;

    // <file>
    let mut file_el = BytesStart::new("file");
    file_el.push_attribute(("source-language", src_lang));
    file_el.push_attribute(("target-language", tgt_lang));
    file_el.push_attribute(("datatype", "plaintext"));
    file_el.push_attribute(("original", "tm-export"));
    file_el.push_attribute(("date", &Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string() as &str));
    xml.write_event(Event::Start(file_el))?;
    xml.write_event(Event::Start(BytesStart::new("body")))?;

    for (i, record) in records.iter().enumerate() {
        let id = format!("{}", i + 1);
        let mut tu = BytesStart::new("trans-unit");
        tu.push_attribute(("id", id.as_str()));
        xml.write_event(Event::Start(tu))?;

        // <source>
        xml.write_event(Event::Start(BytesStart::new("source")))?;
        xml.write_event(Event::Text(BytesText::new(&record.source)))?;
        xml.write_event(Event::End(BytesEnd::new("source")))?;

        // <target>
        xml.write_event(Event::Start(BytesStart::new("target")))?;
        xml.write_event(Event::Text(BytesText::new(&record.target)))?;
        xml.write_event(Event::End(BytesEnd::new("target")))?;

        xml.write_event(Event::End(BytesEnd::new("trans-unit")))?;
    }

    xml.write_event(Event::End(BytesEnd::new("body")))?;
    xml.write_event(Event::End(BytesEnd::new("file")))?;
    xml.write_event(Event::End(BytesEnd::new("xliff")))?;

    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const XLIFF_12: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<xliff version="1.2" xmlns="urn:oasis:names:tc:xliff:document:1.2">
  <file source-language="en" target-language="fr" datatype="plaintext" original="test.txt">
    <body>
      <trans-unit id="1">
        <source>Hello world</source>
        <target>Bonjour monde</target>
      </trans-unit>
      <trans-unit id="2">
        <source>Open file</source>
        <target></target>
      </trans-unit>
      <trans-unit id="3">
        <source>Save document</source>
        <target>Enregistrer le document</target>
      </trans-unit>
    </body>
  </file>
</xliff>"#;

    const XLIFF_20: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<xliff version="2.0" xmlns="urn:oasis:names:tc:xliff:document:2.0"
       srcLang="de" trgLang="ja">
  <file id="f1">
    <unit id="u1">
      <segment>
        <source>Guten Tag</source>
        <target>こんにちは</target>
      </segment>
    </unit>
    <unit id="u2">
      <segment>
        <source>Auf Wiedersehen</source>
        <target>さようなら</target>
      </segment>
    </unit>
  </file>
</xliff>"#;

    #[test]
    fn parse_xliff12_extracts_translated_units() {
        let records = parse_xliff(Cursor::new(XLIFF_12)).unwrap();
        // unit 2 has empty target — should be skipped
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].source, "Hello world");
        assert_eq!(records[0].target, "Bonjour monde");
        assert_eq!(records[0].source_lang, "en");
        assert_eq!(records[0].target_lang, "fr");
    }

    #[test]
    fn parse_xliff12_second_record() {
        let records = parse_xliff(Cursor::new(XLIFF_12)).unwrap();
        assert_eq!(records[1].source, "Save document");
        assert_eq!(records[1].target, "Enregistrer le document");
    }

    #[test]
    fn parse_xliff20_extracts_both_units() {
        let records = parse_xliff(Cursor::new(XLIFF_20)).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].source, "Guten Tag");
        assert_eq!(records[0].target, "こんにちは");
        assert_eq!(records[0].source_lang, "de");
        assert_eq!(records[0].target_lang, "ja");
    }

    #[test]
    fn round_trip_export_import() {
        let original = vec![
            Record::new("Hello", "Hallo", "en", "de"),
            Record::new("Goodbye", "Auf Wiedersehen", "en", "de"),
        ];
        let mut buf = Vec::new();
        write_xliff(&mut buf, &original).unwrap();
        let reimported = parse_xliff(Cursor::new(buf.as_slice())).unwrap();
        assert_eq!(reimported.len(), 2);
        assert_eq!(reimported[0].source, "Hello");
        assert_eq!(reimported[0].target, "Hallo");
        assert_eq!(reimported[1].source, "Goodbye");
        assert_eq!(reimported[1].target, "Auf Wiedersehen");
    }

    #[tokio::test]
    async fn import_to_store() {
        use std::io::Write as IoWrite;
        use tempfile::NamedTempFile;

        let mut f = NamedTempFile::new().unwrap();
        f.write_all(XLIFF_12.as_bytes()).unwrap();

        let store = TmStore::in_memory().await.unwrap();
        let stats = import(f.path(), &store).await.unwrap();

        assert_eq!(stats.imported, 2);
        assert_eq!(stats.skipped_errors, 0);
        assert_eq!(store.count().await.unwrap(), 2);
    }
}
