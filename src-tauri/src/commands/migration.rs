//! One-time migration from Felix 2.x Windows registry settings to the
//! JSON settings file used by Cheshire CAT.
//!
//! Felix 2.x stored settings under:
//!   HKCU\Software\Felix Software\Felix
//!
//! This command reads that key (Windows-only) and returns a partial `Settings`
//! struct that the frontend can preview and then save via `settings_set`.

use serde::Serialize;

/// Legacy settings read from the Felix 2.x registry key.
/// Fields are `Option<T>` because any field may be absent.
#[derive(Debug, Serialize)]
pub struct LegacySettings {
    pub source_lang: Option<String>,
    pub target_lang: Option<String>,
    pub fuzzy_threshold: Option<u8>,
    pub ai_url: Option<String>,
    pub ai_model: Option<String>,
}

/// Read Felix 2.x settings from the Windows registry.
///
/// Returns `Ok(None)` when:
/// - the registry key does not exist (Felix 2.x was never installed), or
/// - the platform is not Windows.
#[tauri::command]
pub async fn migrate_from_felix2() -> Result<Option<LegacySettings>, String> {
    #[cfg(target_os = "windows")]
    {
        read_felix2_registry().map_err(|e| e.to_string())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(None)
    }
}

#[cfg(target_os = "windows")]
fn read_felix2_registry() -> std::io::Result<Option<LegacySettings>> {
    // Read the Felix 2.x registry key via PowerShell.
    // Felix 2.x stored its settings under HKCU\Software\Felix Software\Felix.
    let script = r#"
$ErrorActionPreference = 'Stop'
try {
    $key = 'HKCU:\Software\Felix Software\Felix'
    $props = Get-ItemProperty -Path $key -ErrorAction Stop
    [PSCustomObject]@{
        SourceLang     = $props.SourceLanguage
        TargetLang     = $props.TargetLanguage
        FuzzyThreshold = $props.FuzzyThreshold
        OllamaUrl      = $props.OllamaUrl
        OllamaModel    = $props.OllamaModel
    } | ConvertTo-Json -Compress
} catch {
    'null'
}
"#;

    let output = std::process::Command::new("powershell")
        .args(["-NonInteractive", "-NoProfile", "-Command", script])
        .output()?;

    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if raw == "null" || raw.is_empty() {
        return Ok(None);
    }

    let v: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    Ok(Some(LegacySettings {
        source_lang: v["SourceLang"].as_str().map(str::to_string),
        target_lang: v["TargetLang"].as_str().map(str::to_string),
        fuzzy_threshold: v["FuzzyThreshold"].as_u64().map(|n| n.min(100) as u8),
        ai_url: v["OllamaUrl"].as_str().map(str::to_string),
        ai_model: v["OllamaModel"].as_str().map(str::to_string),
    }))
}
