use crate::error::Result;
use crate::{FileFormat, ImportResult, SourceUnit};
use serde_json::Value;
use std::path::Path;

pub fn import(path: &Path) -> Result<ImportResult> {
    let content = std::fs::read_to_string(path)?;
    import_str(&content)
}

pub fn import_str(content: &str) -> Result<ImportResult> {
    let root: Value = serde_json::from_str(content)?;
    let mut units = Vec::new();
    collect_strings(&root, "", &mut units);
    Ok(ImportResult {
        units,
        source_lang: None,
        target_lang: None,
        format: FileFormat::Json,
    })
}

fn collect_strings(val: &Value, prefix: &str, units: &mut Vec<SourceUnit>) {
    match val {
        Value::String(s) if !s.trim().is_empty() => {
            units.push(SourceUnit {
                id: prefix.to_string(),
                source: s.clone(),
                target: None,
                note: None,
            });
        }
        Value::Object(map) => {
            for (key, child) in map {
                let path = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                collect_strings(child, &path, units);
            }
        }
        Value::Array(arr) => {
            for (i, child) in arr.iter().enumerate() {
                let path = if prefix.is_empty() {
                    i.to_string()
                } else {
                    format!("{prefix}[{i}]")
                };
                collect_strings(child, &path, units);
            }
        }
        _ => {}
    }
}

/// Export units as a flat JSON object `{ "key": "value" }`.
/// The key is the unit id (dot-path); the value is the target if set, else the source.
pub fn export_flat(path: &Path, units: &[SourceUnit]) -> Result<()> {
    let mut map = serde_json::Map::new();
    for unit in units {
        let value = unit.target.as_deref().unwrap_or(&unit.source);
        map.insert(unit.id.clone(), Value::String(value.to_string()));
    }
    let content = serde_json::to_string_pretty(&Value::Object(map))?;
    std::fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FLAT_JSON: &str = r#"{
  "greeting": "Hello world.",
  "farewell": "Goodbye."
}"#;

    const NESTED_JSON: &str = r#"{
  "common": {
    "ok": "OK",
    "cancel": "Cancel"
  },
  "errors": {
    "notFound": "Not found.",
    "forbidden": "Access denied."
  }
}"#;

    const ARRAY_JSON: &str = r#"{
  "items": ["First item.", "Second item.", "Third item."]
}"#;

    const MIXED_JSON: &str = r#"{
  "name": "File Manager",
  "version": 1,
  "features": {
    "search": "Search files",
    "delete": "Delete files"
  }
}"#;

    // ── Basic parsing ──────────────────────────────────────────────────────────

    #[test]
    fn flat_json_two_units() {
        let r = import_str(FLAT_JSON).unwrap();
        assert_eq!(r.units.len(), 2);
    }

    #[test]
    fn flat_json_ids_are_keys() {
        let r = import_str(FLAT_JSON).unwrap();
        let ids: Vec<&str> = r.units.iter().map(|u| u.id.as_str()).collect();
        assert!(ids.contains(&"greeting"));
        assert!(ids.contains(&"farewell"));
    }

    #[test]
    fn flat_json_source_text() {
        let r = import_str(FLAT_JSON).unwrap();
        let greet = r.units.iter().find(|u| u.id == "greeting").unwrap();
        assert_eq!(greet.source, "Hello world.");
    }

    #[test]
    fn nested_json_dot_path_ids() {
        let r = import_str(NESTED_JSON).unwrap();
        assert_eq!(r.units.len(), 4);
        let ids: Vec<&str> = r.units.iter().map(|u| u.id.as_str()).collect();
        assert!(ids.contains(&"common.ok"));
        assert!(ids.contains(&"errors.notFound"));
    }

    #[test]
    fn nested_json_source_text_correct() {
        let r = import_str(NESTED_JSON).unwrap();
        let cancel = r.units.iter().find(|u| u.id == "common.cancel").unwrap();
        assert_eq!(cancel.source, "Cancel");
    }

    #[test]
    fn array_values_use_index_path() {
        let r = import_str(ARRAY_JSON).unwrap();
        assert_eq!(r.units.len(), 3);
        assert!(r.units.iter().any(|u| u.id == "items[0]"));
        assert!(r.units.iter().any(|u| u.id == "items[2]"));
    }

    #[test]
    fn non_string_values_skipped() {
        let r = import_str(MIXED_JSON).unwrap();
        // version: 1 is a number, should be skipped
        assert!(!r.units.iter().any(|u| u.id == "version"));
        // name and two features should be included
        assert_eq!(r.units.len(), 3);
    }

    #[test]
    fn empty_json_object() {
        let r = import_str("{}").unwrap();
        assert!(r.units.is_empty());
    }

    #[test]
    fn format_is_json() {
        let r = import_str(FLAT_JSON).unwrap();
        assert_eq!(r.format, FileFormat::Json);
    }

    #[test]
    fn targets_are_none() {
        let r = import_str(FLAT_JSON).unwrap();
        assert!(r.units.iter().all(|u| u.target.is_none()));
    }

    #[test]
    fn invalid_json_returns_error() {
        assert!(import_str("{not valid json}").is_err());
    }

    // ── Export ────────────────────────────────────────────────────────────────

    #[test]
    fn export_flat_writes_target_values() {
        let units = vec![
            SourceUnit { id: "greeting".into(), source: "Hello.".into(), target: Some("Hallo.".into()), note: None },
            SourceUnit { id: "farewell".into(), source: "Goodbye.".into(), target: None, note: None },
        ];
        let tmp = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
        export_flat(tmp.path(), &units).unwrap();
        let content = std::fs::read_to_string(tmp.path()).unwrap();
        let val: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(val["greeting"].as_str(), Some("Hallo."));
        // No target → falls back to source
        assert_eq!(val["farewell"].as_str(), Some("Goodbye."));
    }
}
