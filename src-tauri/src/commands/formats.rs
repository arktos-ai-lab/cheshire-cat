use std::collections::HashMap;

use cheshire_formats::{
    export_bilingual_docx, export_csv, export_docx_translated, export_json, export_po,
    export_xliff, import, SourceUnit,
};

/// Import a source file (auto-detected format) and return the translation units.
#[tauri::command]
pub async fn format_import(path: String) -> Result<cheshire_formats::ImportResult, String> {
    let p = std::path::Path::new(&path);
    import(p).map_err(|e| e.to_string())
}

/// Export a bilingual XLIFF 1.2 file.
#[tauri::command]
pub async fn format_export_xliff(
    path: String,
    units: Vec<SourceUnit>,
    source_lang: String,
    target_lang: String,
) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    export_xliff(p, &units, &source_lang, &target_lang).map_err(|e| e.to_string())
}

/// Export units as a PO (Gettext) file.
#[tauri::command]
pub async fn format_export_po(path: String, units: Vec<SourceUnit>) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    export_po(p, &units).map_err(|e| e.to_string())
}

/// Export units as a flat JSON object.
#[tauri::command]
pub async fn format_export_json(path: String, units: Vec<SourceUnit>) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    export_json(p, &units).map_err(|e| e.to_string())
}

/// Export units as a UTF-8 CSV file (id, source, target, note).
#[tauri::command]
pub async fn format_export_csv(path: String, units: Vec<SourceUnit>) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    export_csv(p, &units).map_err(|e| e.to_string())
}

/// Export a fresh bilingual DOCX with a two-column Source | Target table.
#[tauri::command]
pub async fn format_export_bilingual_docx(
    path: String,
    units: Vec<SourceUnit>,
    source_lang: String,
    target_lang: String,
) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    export_bilingual_docx(p, &units, &source_lang, &target_lang).map_err(|e| e.to_string())
}

/// Produce a translated DOCX by substituting paragraph translations into the
/// source document.
#[tauri::command]
pub async fn format_export_docx(
    source_path: String,
    dest_path: String,
    translations: HashMap<String, String>,
) -> Result<(), String> {
    let src = std::path::Path::new(&source_path);
    let dst = std::path::Path::new(&dest_path);
    export_docx_translated(src, dst, &translations).map_err(|e| e.to_string())
}
