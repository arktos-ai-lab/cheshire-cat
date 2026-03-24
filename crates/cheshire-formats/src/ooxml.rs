use crate::error::{FormatError, Result};
use crate::{FileFormat, ImportResult, SourceUnit};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::Path;

// ── DOCX ──────────────────────────────────────────────────────────────────────

/// Import a DOCX file. Each non-empty paragraph becomes a SourceUnit.
pub fn import_docx(path: &Path) -> Result<ImportResult> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let xml = read_zip_entry(&mut archive, "word/document.xml")?;
    let units = parse_docx_xml(&xml);

    Ok(ImportResult {
        units,
        source_lang: None,
        target_lang: None,
        format: FileFormat::Docx,
    })
}

fn parse_docx_xml(xml: &str) -> Vec<SourceUnit> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let mut units: Vec<SourceUnit> = Vec::new();
    let mut current_text = String::new();
    let mut in_para = false;
    let mut in_t = false;
    let mut index = 0usize;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local = e.local_name();
                match local.as_ref() {
                    b"p" => {
                        in_para = true;
                        current_text.clear();
                    }
                    b"t" if in_para => {
                        in_t = true;
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                let local = e.local_name();
                if local.as_ref() == b"br" && in_para {
                    current_text.push('\n');
                }
            }
            Ok(Event::Text(ref t)) if in_t => {
                let text = t.unescape().unwrap_or_default();
                current_text.push_str(&text);
            }
            Ok(Event::End(ref e)) => {
                let local = e.local_name();
                match local.as_ref() {
                    b"t" => { in_t = false; }
                    b"p" => {
                        in_para = false;
                        in_t = false;
                        let text = current_text.trim().to_string();
                        if !text.is_empty() {
                            units.push(SourceUnit {
                                id: index.to_string(),
                                source: text,
                                target: None,
                                note: None,
                            });
                            index += 1;
                        }
                        current_text.clear();
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    units
}

// ── XLSX ──────────────────────────────────────────────────────────────────────

/// Import an XLSX file. Each unique string cell in the shared strings table
/// becomes a SourceUnit. Numbers and formulas are skipped.
pub fn import_xlsx(path: &Path) -> Result<ImportResult> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let strings = read_shared_strings(&mut archive)?;
    let units = strings
        .into_iter()
        .enumerate()
        .filter(|(_, s)| !s.trim().is_empty())
        .map(|(i, s)| SourceUnit {
            id: i.to_string(),
            source: s,
            target: None,
            note: None,
        })
        .collect();

    Ok(ImportResult {
        units,
        source_lang: None,
        target_lang: None,
        format: FileFormat::Xlsx,
    })
}

fn read_shared_strings<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> Result<Vec<String>> {
    // Some XLSX files have no sharedStrings.xml (all cells are inline values)
    let entry = match archive.by_name("xl/sharedStrings.xml") {
        Ok(e) => e,
        Err(zip::result::ZipError::FileNotFound) => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };
    let xml = std::io::read_to_string(entry)?;

    let mut reader = Reader::from_str(&xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut strings: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_t = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if e.local_name().as_ref() == b"t" {
                    in_t = true;
                    current.clear();
                } else if e.local_name().as_ref() == b"si" {
                    current.clear();
                }
            }
            Ok(Event::Text(ref t)) if in_t => {
                let text = t.unescape().unwrap_or_default();
                current.push_str(&text);
            }
            Ok(Event::End(ref e)) => {
                match e.local_name().as_ref() {
                    b"t" => { in_t = false; }
                    b"si" => {
                        strings.push(current.clone());
                        current.clear();
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(strings)
}

// ── DOCX translation export ───────────────────────────────────────────────────

/// Produce a translated copy of a DOCX file.
///
/// `translations` maps each source paragraph text (trimmed) to its translation.
/// Paragraphs with no matching translation are copied unchanged, preserving the
/// full ZIP structure (styles, images, relationships, etc.).
pub fn export_docx_translated(
    source_path: &Path,
    dest_path: &Path,
    translations: &HashMap<String, String>,
) -> Result<()> {
    let source_file = std::fs::File::open(source_path)?;
    let mut source_archive = zip::ZipArchive::new(source_file)?;

    let original_xml = read_zip_entry(&mut source_archive, "word/document.xml")?;
    let translated_xml = apply_paragraph_translations(&original_xml, translations);

    let dest_file = std::fs::File::create(dest_path)?;
    let mut dest_writer = zip::ZipWriter::new(dest_file);

    for i in 0..source_archive.len() {
        let mut entry = source_archive.by_index(i)?;
        let name = entry.name().to_string();
        let method = entry.compression();
        let options = zip::write::SimpleFileOptions::default().compression_method(method);

        dest_writer.start_file(&name, options)?;
        if name == "word/document.xml" {
            dest_writer.write_all(translated_xml.as_bytes())?;
        } else {
            std::io::copy(&mut entry, &mut dest_writer)?;
        }
    }

    dest_writer.finish()?;
    Ok(())
}

/// Replace the text content of paragraphs that have a matching translation.
/// Paragraphs without a translation are emitted verbatim.
fn apply_paragraph_translations(xml: &str, translations: &HashMap<String, String>) -> String {
    let paragraphs = extract_paragraph_data(xml);
    let mut result = String::with_capacity(xml.len());
    let mut pos = 0usize;

    for (start, end, text) in paragraphs {
        result.push_str(&xml[pos..start]);
        if let Some(translation) = translations.get(&text) {
            result.push_str("<w:p><w:r><w:t xml:space=\"preserve\">");
            result.push_str(&xml_escape(translation));
            result.push_str("</w:t></w:r></w:p>");
        } else {
            result.push_str(&xml[start..end]);
        }
        pos = end;
    }

    result.push_str(&xml[pos..]);
    result
}

/// Return `(start_byte, end_byte, trimmed_text)` for each non-empty paragraph.
///
/// Byte positions are into the original `xml` slice so that callers can
/// reconstruct the document by slicing around them.
fn extract_paragraph_data(xml: &str) -> Vec<(usize, usize, String)> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let mut results = Vec::new();

    let mut para_start = 0u64;
    let mut in_para = false;
    let mut in_t = false;
    let mut current_text = String::new();

    loop {
        let before = reader.buffer_position();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local = e.local_name();
                match local.as_ref() {
                    b"p" if !in_para => {
                        in_para = true;
                        current_text.clear();
                        para_start = before;
                    }
                    b"t" if in_para => {
                        in_t = true;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref t)) if in_t => {
                current_text.push_str(&t.unescape().unwrap_or_default());
            }
            Ok(Event::End(ref e)) => {
                let local = e.local_name();
                match local.as_ref() {
                    b"t" => {
                        in_t = false;
                    }
                    b"p" if in_para => {
                        let para_end = reader.buffer_position();
                        let text = current_text.trim().to_string();
                        if !text.is_empty() {
                            results.push((para_start as usize, para_end as usize, text));
                        }
                        in_para = false;
                        in_t = false;
                        current_text.clear();
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    results
}

// ── Bilingual DOCX export ─────────────────────────────────────────────────────

/// Create a fresh DOCX containing a two-column (Source | Target) table.
///
/// Each `SourceUnit` becomes one row.  Units whose `target` is `None` get an
/// empty target cell so the translator can fill them in later.
pub fn export_docx_bilingual(
    path: &Path,
    units: &[SourceUnit],
    source_lang: &str,
    target_lang: &str,
) -> Result<()> {
    let file = std::fs::File::create(path)?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // [Content_Types].xml
    zip.start_file("[Content_Types].xml", opts)?;
    zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
<Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
<Default Extension=\"xml\" ContentType=\"application/xml\"/>\
<Override PartName=\"/word/document.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>\
</Types>")?;

    // _rels/.rels
    zip.start_file("_rels/.rels", opts)?;
    zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"word/document.xml\"/>\
</Relationships>")?;

    // word/_rels/document.xml.rels
    zip.start_file("word/_rels/document.xml.rels", opts)?;
    zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
</Relationships>")?;

    // word/document.xml
    zip.start_file("word/document.xml", opts)?;
    let doc_xml = build_bilingual_document_xml(units, source_lang, target_lang);
    zip.write_all(doc_xml.as_bytes())?;

    zip.finish()?;
    Ok(())
}

fn build_bilingual_document_xml(units: &[SourceUnit], source_lang: &str, target_lang: &str) -> String {
    let mut out = String::with_capacity(units.len() * 200 + 1024);
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>");
    out.push_str("<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">");
    out.push_str("<w:body>");
    out.push_str("<w:tbl>");
    // Table properties: full-width, simple borders
    out.push_str("<w:tblPr>\
<w:tblW w:w=\"0\" w:type=\"auto\"/>\
<w:tblBorders>\
<w:top w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"auto\"/>\
<w:left w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"auto\"/>\
<w:bottom w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"auto\"/>\
<w:right w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"auto\"/>\
<w:insideH w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"auto\"/>\
<w:insideV w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"auto\"/>\
</w:tblBorders>\
</w:tblPr>");
    out.push_str("<w:tblGrid><w:gridCol w:w=\"4676\"/><w:gridCol w:w=\"4676\"/></w:tblGrid>");

    // Header row
    let src_header = format!("Source ({})", source_lang);
    let tgt_header = format!("Target ({})", target_lang);
    out.push_str("<w:tr>");
    bilingual_bold_cell(&mut out, &src_header);
    bilingual_bold_cell(&mut out, &tgt_header);
    out.push_str("</w:tr>");

    // Data rows
    for unit in units {
        let target = unit.target.as_deref().unwrap_or("");
        out.push_str("<w:tr>");
        bilingual_cell(&mut out, &unit.source);
        bilingual_cell(&mut out, target);
        out.push_str("</w:tr>");
    }

    out.push_str("</w:tbl>");
    out.push_str("</w:body>");
    out.push_str("</w:document>");
    out
}

fn bilingual_cell(out: &mut String, text: &str) {
    out.push_str("<w:tc><w:p><w:r><w:t xml:space=\"preserve\">");
    out.push_str(&xml_escape(text));
    out.push_str("</w:t></w:r></w:p></w:tc>");
}

fn bilingual_bold_cell(out: &mut String, text: &str) {
    out.push_str("<w:tc><w:p><w:r><w:rPr><w:b/></w:rPr><w:t xml:space=\"preserve\">");
    out.push_str(&xml_escape(text));
    out.push_str("</w:t></w:r></w:p></w:tc>");
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn read_zip_entry<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    name: &str,
) -> Result<String> {
    let entry = archive.by_name(name).map_err(|e| {
        if matches!(e, zip::result::ZipError::FileNotFound) {
            FormatError::Malformed {
                format: "OOXML",
                reason: format!("missing entry: {name}"),
            }
        } else {
            e.into()
        }
    })?;
    Ok(std::io::read_to_string(entry)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── DOCX XML parsing ──────────────────────────────────────────────────────

    const DOCX_XML_SIMPLE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:r><w:t>Hello world.</w:t></w:r>
    </w:p>
    <w:p>
      <w:r><w:t>Please sign the document.</w:t></w:r>
    </w:p>
  </w:body>
</w:document>"#;

    const DOCX_XML_MULTIRUN: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:r><w:t xml:space="preserve">Hello </w:t></w:r>
      <w:r><w:t>world</w:t></w:r>
      <w:r><w:t>.</w:t></w:r>
    </w:p>
  </w:body>
</w:document>"#;

    const DOCX_XML_EMPTY_PARA: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:r><w:t>First.</w:t></w:r></w:p>
    <w:p></w:p>
    <w:p><w:r><w:t>Second.</w:t></w:r></w:p>
  </w:body>
</w:document>"#;

    const DOCX_XML_UNICODE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:r><w:t>書類に署名してください。</w:t></w:r>
    </w:p>
  </w:body>
</w:document>"#;

    #[test]
    fn docx_xml_two_paragraphs() {
        let units = parse_docx_xml(DOCX_XML_SIMPLE);
        assert_eq!(units.len(), 2);
    }

    #[test]
    fn docx_xml_source_text_correct() {
        let units = parse_docx_xml(DOCX_XML_SIMPLE);
        assert_eq!(units[0].source, "Hello world.");
    }

    #[test]
    fn docx_xml_multirun_joined() {
        let units = parse_docx_xml(DOCX_XML_MULTIRUN);
        assert_eq!(units.len(), 1);
        assert_eq!(units[0].source, "Hello world.");
    }

    #[test]
    fn docx_xml_empty_paragraphs_skipped() {
        let units = parse_docx_xml(DOCX_XML_EMPTY_PARA);
        assert_eq!(units.len(), 2);
        assert_eq!(units[0].source, "First.");
        assert_eq!(units[1].source, "Second.");
    }

    #[test]
    fn docx_xml_unicode_preserved() {
        let units = parse_docx_xml(DOCX_XML_UNICODE);
        assert_eq!(units.len(), 1);
        assert_eq!(units[0].source, "書類に署名してください。");
    }

    #[test]
    fn docx_xml_sequential_indices() {
        let units = parse_docx_xml(DOCX_XML_SIMPLE);
        assert_eq!(units[0].id, "0");
        assert_eq!(units[1].id, "1");
    }

    #[test]
    fn docx_xml_targets_are_none() {
        let units = parse_docx_xml(DOCX_XML_SIMPLE);
        assert!(units.iter().all(|u| u.target.is_none()));
    }

    #[test]
    fn docx_xml_empty_document() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body></w:body>
</w:document>"#;
        let units = parse_docx_xml(xml);
        assert!(units.is_empty());
    }

    // ── Shared strings ────────────────────────────────────────────────────────

    const SHARED_STRINGS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="3" uniqueCount="3">
  <si><t>Contract Title</t></si>
  <si><t>Penalty Clause</t></si>
  <si><t>Effective Date</t></si>
</sst>"#;

    fn parse_shared_strings_from_str(xml: &str) -> Vec<String> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();
        let mut strings = Vec::new();
        let mut current = String::new();
        let mut in_t = false;
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    if e.local_name().as_ref() == b"t" { in_t = true; current.clear(); }
                    else if e.local_name().as_ref() == b"si" { current.clear(); }
                }
                Ok(Event::Text(ref t)) if in_t => {
                    current.push_str(&t.unescape().unwrap_or_default());
                }
                Ok(Event::End(ref e)) => match e.local_name().as_ref() {
                    b"t" => { in_t = false; }
                    b"si" => { strings.push(current.clone()); current.clear(); }
                    _ => {}
                },
                Ok(Event::Eof) => break,
                _ => {}
            }
            buf.clear();
        }
        strings
    }

    #[test]
    fn shared_strings_parsed_correctly() {
        let strings = parse_shared_strings_from_str(SHARED_STRINGS_XML);
        assert_eq!(strings.len(), 3);
        assert_eq!(strings[0], "Contract Title");
        assert_eq!(strings[1], "Penalty Clause");
        assert_eq!(strings[2], "Effective Date");
    }

    // ── Format detection ──────────────────────────────────────────────────────

    #[test]
    fn nonexistent_docx_returns_error() {
        assert!(import_docx(Path::new("/nonexistent/file.docx")).is_err());
    }

    #[test]
    fn nonexistent_xlsx_returns_error() {
        assert!(import_xlsx(Path::new("/nonexistent/file.xlsx")).is_err());
    }

    // ── DOCX translation export ───────────────────────────────────────────────

    #[test]
    fn apply_translations_replaces_matching_paragraphs() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:r><w:t>Hello world.</w:t></w:r></w:p>
    <w:p><w:r><w:t>Please sign the document.</w:t></w:r></w:p>
  </w:body>
</w:document>"#;

        let mut translations = HashMap::new();
        translations.insert("Hello world.".to_string(), "Hallo Welt.".to_string());

        let result = apply_paragraph_translations(xml, &translations);
        assert!(result.contains("Hallo Welt."), "translated para should appear");
        assert!(result.contains("Please sign the document."), "untranslated para should remain");
        assert!(!result.contains("<w:t>Hello world.</w:t>"), "original should be replaced");
    }

    #[test]
    fn apply_translations_untranslated_para_preserved_verbatim() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body><w:p><w:r><w:t>Keep me.</w:t></w:r></w:p></w:body>
</w:document>"#;
        let translations = HashMap::new(); // empty
        let result = apply_paragraph_translations(xml, &translations);
        assert!(result.contains("Keep me."));
    }

    #[test]
    fn apply_translations_xml_escapes_special_chars() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body><w:p><w:r><w:t>Hello.</w:t></w:r></w:p></w:body>
</w:document>"#;
        let mut translations = HashMap::new();
        translations.insert("Hello.".to_string(), "Say <hi> & \"bye\".".to_string());
        let result = apply_paragraph_translations(xml, &translations);
        assert!(result.contains("&lt;hi&gt;"));
        assert!(result.contains("&amp;"));
    }

    #[test]
    fn extract_paragraph_data_finds_correct_text() {
        let data = extract_paragraph_data(DOCX_XML_SIMPLE);
        assert_eq!(data.len(), 2);
        assert_eq!(data[0].2, "Hello world.");
        assert_eq!(data[1].2, "Please sign the document.");
    }

    #[test]
    fn extract_paragraph_data_byte_positions_valid() {
        let xml = DOCX_XML_SIMPLE;
        let data = extract_paragraph_data(xml);
        for (start, end, text) in &data {
            // The byte range should sit inside the XML string
            assert!(*end <= xml.len(), "end byte out of range");
            assert!(*start < *end, "start >= end");
            // The text should appear somewhere in that slice
            let slice = &xml[*start..*end];
            assert!(slice.contains(text.as_str()), "text not in byte range");
        }
    }
}
