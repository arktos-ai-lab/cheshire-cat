use cheshire_ai::AiMode;
use serde::{Deserialize, Serialize};

/// Persisted application settings, stored as JSON in the app data directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// BCP 47 source language tag (e.g. "en").
    pub source_lang: String,
    /// BCP 47 target language tag (e.g. "ja").
    pub target_lang: String,
    /// Minimum fuzzy match score to include in results (0–100, default 70).
    pub fuzzy_threshold: u8,
    /// Maximum number of TM matches to return per lookup (default 5).
    pub max_matches: usize,
    /// AI backend base URL (default "http://localhost:11434" for Ollama).
    pub ai_url: String,
    /// AI model identifier (e.g. "llama3.2:3b", "gpt-4o-mini").
    pub ai_model: String,
    /// Which AI backend to use.
    #[serde(default)]
    pub ai_mode: AiMode,
    /// Bearer token for cloud AI APIs. Empty string means no token.
    #[serde(default)]
    pub ai_api_key: String,
    /// Optional absolute path to the TM SQLite file.
    /// Defaults to `<app_data>/tm.db` when `None`.
    pub tm_db_path: Option<String>,
    /// Read-only: `true` on Windows, `false` on macOS/Linux.
    /// Not persisted — injected at runtime so the frontend can hide
    /// Windows-only features (Office COM bridge, Felix 2.x migration).
    #[serde(default)]
    pub is_windows: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            source_lang: String::new(),
            target_lang: String::new(),
            fuzzy_threshold: 70,
            max_matches: 5,
            ai_url: "http://localhost:11434".into(),
            ai_model: "llama3.2:3b".into(),
            ai_mode: AiMode::Ollama,
            ai_api_key: String::new(),
            tm_db_path: None,
            is_windows: false, // overwritten at runtime in settings_get
        }
    }
}
