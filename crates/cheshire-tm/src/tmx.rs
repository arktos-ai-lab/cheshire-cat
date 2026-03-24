//! TMX (Translation Memory eXchange) import and export.
//!
//! Supports TMX 1.4b — the version used by Felix and the current industry standard.

use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::{fs::File, str};

use chrono::Utc;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};

use crate::{
    error::Result,
    record::{ImportStats, Record, RecordMetadata},
    store::TmStore,
};

/// Import records from a TMX file into the store.
pub async fn import(path: &Path, store: &TmStore) -> Result<ImportStats> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let records = parse_tmx(reader)?;

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

/// Export records to a TMX file.
pub async fn export(path: &Path, records: &[Record]) -> Result<()> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    write_tmx(writer, records)
}

// ── Parser ────────────────────────────────────────────────────────────────────

fn parse_tmx(reader: impl BufRead) -> Result<Vec<Record>> {
    let mut xml = Reader::from_reader(reader);
    xml.config_mut().trim_text(true);

    let mut records: Vec<Record> = Vec::new();
    let mut buf = Vec::new();

    // Parser state
    let mut in_tu = false;
    let mut in_tuv = false;
    let mut in_seg = false;
    let mut current_lang: Option<String> = None;
    let mut source_text: Option<(String, String)> = None; // (lang, text)
    let mut target_text: Option<(String, String)> = None;
    let mut creator: Option<String> = None;

    loop {
        match xml.read_event_into(&mut buf)? {
            Event::Start(e) | Event::Empty(e) => match e.name().as_ref() {
                b"tu" => {
                    in_tu = true;
                    source_text = None;
                    target_text = None;
                    creator = None;
                    // Extract creationid if present
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"creationid" {
                            creator = Some(
                                String::from_utf8_lossy(&attr.value).into_owned(),
                            );
                        }
                    }
                }
                b"tuv" if in_tu => {
                    in_tuv = true;
                    // Extract xml:lang attribute
                    for attr in e.attributes().flatten() {
                        let key = attr.key.as_ref();
                        if key == b"xml:lang" || key == b"lang" {
                            current_lang = Some(
                                String::from_utf8_lossy(&attr.value)
                                    .to_lowercase()
                                    .replace('_', "-"),
                            );
                        }
                    }
                }
                b"seg" if in_tuv => {
                    in_seg = true;
                }
                _ => {}
            },
            Event::Text(e) if in_seg => {
                let text = e.unescape()?.into_owned();
                if let Some(lang) = &current_lang {
                    // Simple heuristic: first two TUVs encountered are source/target.
                    // We just accumulate both; pairing happens at </tu>.
                    if source_text.is_none() {
                        source_text = Some((lang.clone(), text));
                    } else if target_text.is_none() && source_text.as_ref().map(|(l, _)| l) != Some(lang) {
                        target_text = Some((lang.clone(), text));
                    }
                }
            }
            Event::End(e) => match e.name().as_ref() {
                b"seg" => in_seg = false,
                b"tuv" => {
                    in_tuv = false;
                    current_lang = None;
                }
                b"tu" => {
                    in_tu = false;
                    if let (Some((src_lang, src)), Some((tgt_lang, tgt))) =
                        (source_text.take(), target_text.take())
                    {
                        let mut record = Record::new(src, tgt, src_lang, tgt_lang);
                        record.metadata = RecordMetadata {
                            creator: creator.clone(),
                            ..Default::default()
                        };
                        records.push(record);
                    }
                }
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(records)
}

// ── Writer ────────────────────────────────────────────────────────────────────

fn write_tmx(writer: impl Write, records: &[Record]) -> Result<()> {
    let mut xml = Writer::new_with_indent(writer, b' ', 2);

    // XML declaration
    xml.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

    // <tmx version="1.4">
    let mut tmx_start = BytesStart::new("tmx");
    tmx_start.push_attribute(("version", "1.4"));
    xml.write_event(Event::Start(tmx_start))?;

    // <header>
    let mut header = BytesStart::new("header");
    header.push_attribute(("creationtool", "Cheshire CAT"));
    header.push_attribute(("creationtoolversion", env!("CARGO_PKG_VERSION")));
    header.push_attribute(("datatype", "plaintext"));
    header.push_attribute(("segtype", "sentence"));
    header.push_attribute(("adminlang", "en"));
    header.push_attribute(("creationdate", &Utc::now().format("%Y%m%dT%H%M%SZ").to_string() as &str));
    xml.write_event(Event::Empty(header))?;

    // <body>
    xml.write_event(Event::Start(BytesStart::new("body")))?;

    for record in records {
        // <tu>
        let mut tu = BytesStart::new("tu");
        if let Some(creator) = &record.metadata.creator {
            tu.push_attribute(("creationid", creator.as_str()));
        }
        tu.push_attribute(("creationdate", &record.created_at.format("%Y%m%dT%H%M%SZ").to_string() as &str));
        xml.write_event(Event::Start(tu))?;

        write_tuv(&mut xml, &record.source_lang, &record.source)?;
        write_tuv(&mut xml, &record.target_lang, &record.target)?;

        xml.write_event(Event::End(BytesEnd::new("tu")))?;
    }

    xml.write_event(Event::End(BytesEnd::new("body")))?;
    xml.write_event(Event::End(BytesEnd::new("tmx")))?;

    Ok(())
}

fn write_tuv(xml: &mut Writer<impl Write>, lang: &str, text: &str) -> Result<()> {
    let mut tuv = BytesStart::new("tuv");
    tuv.push_attribute(("xml:lang", lang));
    xml.write_event(Event::Start(tuv))?;
    xml.write_event(Event::Start(BytesStart::new("seg")))?;
    xml.write_event(Event::Text(BytesText::new(text)))?;
    xml.write_event(Event::End(BytesEnd::new("seg")))?;
    xml.write_event(Event::End(BytesEnd::new("tuv")))?;
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const SAMPLE_TMX: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<tmx version="1.4">
  <header creationtool="TestTool" datatype="plaintext" segtype="sentence" adminlang="en"/>
  <body>
    <tu creationid="translator1">
      <tuv xml:lang="en"><seg>Hello world</seg></tuv>
      <tuv xml:lang="ja"><seg>こんにちは世界</seg></tuv>
    </tu>
    <tu>
      <tuv xml:lang="en"><seg>Please sign the document.</seg></tuv>
      <tuv xml:lang="ja"><seg>書類に署名してください。</seg></tuv>
    </tu>
  </body>
</tmx>"#;

    #[test]
    fn parse_sample_tmx() {
        let records = parse_tmx(Cursor::new(SAMPLE_TMX)).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].source, "Hello world");
        assert_eq!(records[0].target, "こんにちは世界");
        assert_eq!(records[0].source_lang, "en");
        assert_eq!(records[0].target_lang, "ja");
        assert_eq!(records[0].metadata.creator.as_deref(), Some("translator1"));
    }

    #[test]
    fn round_trip_tmx() {
        let records = parse_tmx(Cursor::new(SAMPLE_TMX)).unwrap();
        let mut buf = Vec::new();
        write_tmx(&mut buf, &records).unwrap();
        let exported = String::from_utf8(buf).unwrap();

        // Re-parse the exported TMX
        let re_parsed = parse_tmx(Cursor::new(exported.as_str())).unwrap();
        assert_eq!(re_parsed.len(), 2);
        assert_eq!(re_parsed[0].source, records[0].source);
        assert_eq!(re_parsed[0].target, records[0].target);
    }

    #[tokio::test]
    async fn import_to_store() {
        use tempfile::NamedTempFile;
        use std::io::Write as IoWrite;

        let mut tmx_file = NamedTempFile::new().unwrap();
        tmx_file.write_all(SAMPLE_TMX.as_bytes()).unwrap();

        let store = TmStore::in_memory().await.unwrap();
        let stats = store.import_tmx(tmx_file.path()).await.unwrap();

        assert_eq!(stats.imported, 2);
        assert_eq!(stats.skipped_errors, 0);
        assert_eq!(store.count().await.unwrap(), 2);
    }
}
