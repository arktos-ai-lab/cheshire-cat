pub mod error;
pub mod xliff;
pub mod ooxml;
pub mod html;
pub mod plaintext;
pub mod po;
pub mod json;
pub mod csv;

pub use error::{FormatError, Result};

use std::path::Path;
use serde::{Deserialize, Serialize};

/// A single translatable unit extracted from a source document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SourceUnit {
    /// Stable identifier within the document (e.g. trans-unit id, paragraph index).
    pub id: String,
    /// Source text (may contain inline tag placeholders like `{1}`).
    pub source: String,
    /// Pre-existing translation, if present in the file.
    pub target: Option<String>,
    /// Translator note or context.
    pub note: Option<String>,
}

/// Result of importing a source document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub units: Vec<SourceUnit>,
    pub source_lang: Option<String>,
    pub target_lang: Option<String>,
    pub format: FileFormat,
}

/// The file format that was imported or will be exported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileFormat {
    Xliff12,
    Xliff20,
    Docx,
    Xlsx,
    Html,
    PlainText,
    Po,
    Json,
    Csv,
}

/// Detect the file format from the file extension.
pub fn detect_format(path: &Path) -> Result<FileFormat> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "xlf" | "xliff" => Ok(FileFormat::Xliff12), // version detected inside
        "docx" => Ok(FileFormat::Docx),
        "xlsx" => Ok(FileFormat::Xlsx),
        "htm" | "html" => Ok(FileFormat::Html),
        "txt" => Ok(FileFormat::PlainText),
        "po" | "pot" => Ok(FileFormat::Po),
        "json" => Ok(FileFormat::Json),
        "csv" => Ok(FileFormat::Csv),
        _ => Err(FormatError::UnknownExtension { ext }),
    }
}

/// Import a source document from `path`. Format is auto-detected from extension.
pub fn import(path: &Path) -> Result<ImportResult> {
    let fmt = detect_format(path)?;
    match fmt {
        FileFormat::Xliff12 | FileFormat::Xliff20 => xliff::import(path),
        FileFormat::Docx => ooxml::import_docx(path),
        FileFormat::Xlsx => ooxml::import_xlsx(path),
        FileFormat::Html => html::import(path),
        FileFormat::PlainText => plaintext::import(path),
        FileFormat::Po => po::import(path),
        FileFormat::Json => json::import(path),
        FileFormat::Csv => csv::import(path),
    }
}

/// Export a bilingual XLIFF 1.2 file containing source + (optional) translations.
pub fn export_xliff(
    path: &Path,
    units: &[SourceUnit],
    source_lang: &str,
    target_lang: &str,
) -> Result<()> {
    xliff::export(path, units, source_lang, target_lang)
}

/// Export units as a PO (Gettext) file.
pub fn export_po(path: &Path, units: &[SourceUnit]) -> Result<()> {
    po::export(path, units)
}

/// Export units as a flat JSON object (`{ "key": "translated value" }`).
pub fn export_json(path: &Path, units: &[SourceUnit]) -> Result<()> {
    json::export_flat(path, units)
}

/// Export units as a UTF-8 CSV file with columns: id, source, target, note.
pub fn export_csv(path: &Path, units: &[SourceUnit]) -> Result<()> {
    csv::export(path, units)
}

/// Create a fresh bilingual DOCX with a two-column Source | Target table.
pub fn export_bilingual_docx(
    path: &Path,
    units: &[SourceUnit],
    source_lang: &str,
    target_lang: &str,
) -> Result<()> {
    ooxml::export_docx_bilingual(path, units, source_lang, target_lang)
}

/// Produce a translated DOCX file from a source DOCX and a map of
/// `source_paragraph_text → translation`.
pub fn export_docx_translated(
    source_path: &Path,
    dest_path: &Path,
    translations: &std::collections::HashMap<String, String>,
) -> Result<()> {
    ooxml::export_docx_translated(source_path, dest_path, translations)
}
