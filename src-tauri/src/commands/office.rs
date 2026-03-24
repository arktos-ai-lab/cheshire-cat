//! Windows COM bridge — communicates with the active Word/Excel/PowerPoint
//! instance via PowerShell's built-in COM support.
//!
//! All commands degrade gracefully on non-Windows platforms, returning
//! `Ok(None)` so the frontend can hide the Office panel.

// ── Get active Word selection ──────────────────────────────────────────────────

/// Returns the currently selected text in the active Microsoft Word document.
///
/// Returns `Ok(None)` when no Word instance is running, nothing is selected,
/// or the platform is not Windows.
#[tauri::command]
pub async fn office_get_word_selection() -> Result<Option<String>, String> {
    #[cfg(target_os = "windows")]
    {
        get_word_selection_impl().map_err(|e| e.to_string())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(None)
    }
}

#[cfg(target_os = "windows")]
fn get_word_selection_impl() -> std::io::Result<Option<String>> {
    // Use PowerShell's native COM support to read the Word selection.
    // GetActiveObject returns the running instance without launching a new one.
    let script = r#"
$ErrorActionPreference = 'Stop'
try {
    $word = [System.Runtime.InteropServices.Marshal]::GetActiveObject('Word.Application')
    $sel  = $word.Selection.Text
    # Word appends \r to each paragraph; normalize to nothing for single-line
    $sel.TrimEnd([char]13, [char]10, [char]7)
} catch {
    ''
}
"#;
    let output = std::process::Command::new("powershell")
        .args(["-NonInteractive", "-NoProfile", "-Command", script])
        .output()?;

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        Ok(None)
    } else {
        Ok(Some(text))
    }
}

// ── Insert text into active Word document ──────────────────────────────────────

/// Inserts `text` at the current cursor position in the active Microsoft Word
/// document, replacing the current selection if any.
///
/// Returns an error string when Word is not running or the platform is not
/// Windows.
#[tauri::command]
pub async fn office_insert_into_word(text: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        insert_word_text_impl(&text).map_err(|e| e.to_string())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = text;
        Err("Office COM bridge is only available on Windows.".into())
    }
}

#[cfg(target_os = "windows")]
fn insert_word_text_impl(text: &str) -> std::io::Result<()> {
    // Escape single quotes in the text for embedding in a PowerShell string.
    let escaped = text.replace('\'', "''");
    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$word = [System.Runtime.InteropServices.Marshal]::GetActiveObject('Word.Application')
$word.Selection.TypeText('{escaped}')
"#
    );
    let status = std::process::Command::new("powershell")
        .args(["-NonInteractive", "-NoProfile", "-Command", &script])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("PowerShell returned non-zero exit code"))
    }
}

// ── Check if Word is running ───────────────────────────────────────────────────

/// Returns `true` when a Microsoft Word instance is currently running.
/// Always returns `false` on non-Windows platforms.
#[tauri::command]
pub async fn office_word_is_running() -> bool {
    #[cfg(target_os = "windows")]
    {
        app_is_running_impl("Word.Application")
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

// ── Excel COM bridge ───────────────────────────────────────────────────────────

/// Returns the value of the currently selected cell(s) in Excel.
/// Multiple selected cells are joined with newlines.
#[tauri::command]
pub async fn office_get_excel_selection() -> Result<Option<String>, String> {
    #[cfg(target_os = "windows")]
    {
        get_excel_selection_impl().map_err(|e| e.to_string())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(None)
    }
}

#[cfg(target_os = "windows")]
fn get_excel_selection_impl() -> std::io::Result<Option<String>> {
    let script = r#"
$ErrorActionPreference = 'Stop'
try {
    $excel = [System.Runtime.InteropServices.Marshal]::GetActiveObject('Excel.Application')
    $cells = @()
    foreach ($cell in $excel.Selection.Cells) {
        $v = $cell.Value2
        if ($v -ne $null) { $cells += [string]$v }
    }
    $cells -join "`n"
} catch {
    ''
}
"#;
    let output = std::process::Command::new("powershell")
        .args(["-NonInteractive", "-NoProfile", "-Command", script])
        .output()?;
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() { Ok(None) } else { Ok(Some(text)) }
}

/// Writes `text` into the active Excel cell.
#[tauri::command]
pub async fn office_insert_into_excel(text: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        insert_excel_text_impl(&text).map_err(|e| e.to_string())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = text;
        Err("Office COM bridge is only available on Windows.".into())
    }
}

#[cfg(target_os = "windows")]
fn insert_excel_text_impl(text: &str) -> std::io::Result<()> {
    let escaped = text.replace('\'', "''");
    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$excel = [System.Runtime.InteropServices.Marshal]::GetActiveObject('Excel.Application')
$excel.ActiveCell.Value2 = '{escaped}'
"#
    );
    let status = std::process::Command::new("powershell")
        .args(["-NonInteractive", "-NoProfile", "-Command", &script])
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("PowerShell returned non-zero exit code"))
    }
}

/// Returns `true` when Microsoft Excel is currently running.
#[tauri::command]
pub async fn office_excel_is_running() -> bool {
    #[cfg(target_os = "windows")]
    {
        app_is_running_impl("Excel.Application")
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

// ── PowerPoint COM bridge ──────────────────────────────────────────────────────

/// Returns the selected text in the active PowerPoint presentation.
#[tauri::command]
pub async fn office_get_ppt_selection() -> Result<Option<String>, String> {
    #[cfg(target_os = "windows")]
    {
        get_ppt_selection_impl().map_err(|e| e.to_string())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(None)
    }
}

#[cfg(target_os = "windows")]
fn get_ppt_selection_impl() -> std::io::Result<Option<String>> {
    let script = r#"
$ErrorActionPreference = 'Stop'
try {
    $ppt = [System.Runtime.InteropServices.Marshal]::GetActiveObject('PowerPoint.Application')
    $ppt.ActiveWindow.Selection.TextRange.Text.Trim()
} catch {
    ''
}
"#;
    let output = std::process::Command::new("powershell")
        .args(["-NonInteractive", "-NoProfile", "-Command", script])
        .output()?;
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() { Ok(None) } else { Ok(Some(text)) }
}

/// Replaces the selected text in PowerPoint with `text`.
#[tauri::command]
pub async fn office_insert_into_ppt(text: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        insert_ppt_text_impl(&text).map_err(|e| e.to_string())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = text;
        Err("Office COM bridge is only available on Windows.".into())
    }
}

#[cfg(target_os = "windows")]
fn insert_ppt_text_impl(text: &str) -> std::io::Result<()> {
    let escaped = text.replace('\'', "''");
    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$ppt = [System.Runtime.InteropServices.Marshal]::GetActiveObject('PowerPoint.Application')
$ppt.ActiveWindow.Selection.TextRange.Text = '{escaped}'
"#
    );
    let status = std::process::Command::new("powershell")
        .args(["-NonInteractive", "-NoProfile", "-Command", &script])
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("PowerShell returned non-zero exit code"))
    }
}

/// Returns `true` when Microsoft PowerPoint is currently running.
#[tauri::command]
pub async fn office_ppt_is_running() -> bool {
    #[cfg(target_os = "windows")]
    {
        app_is_running_impl("PowerPoint.Application")
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

// ── Shared helper ──────────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
fn app_is_running_impl(prog_id: &str) -> bool {
    let script = format!(
        r#"
try {{
    [System.Runtime.InteropServices.Marshal]::GetActiveObject('{prog_id}') | Out-Null
    'true'
}} catch {{
    'false'
}}
"#
    );
    std::process::Command::new("powershell")
        .args(["-NonInteractive", "-NoProfile", "-Command", &script])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "true")
        .unwrap_or(false)
}
