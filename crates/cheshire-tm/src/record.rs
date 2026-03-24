use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A single translation unit: source segment + target translation + metadata.
/// Mirrors Felix's original Record type but with cross-platform SQLite backing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Record {
    pub id: Uuid,
    pub source: String,
    pub target: String,
    pub source_lang: String,
    pub target_lang: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub metadata: RecordMetadata,
}

impl Record {
    pub fn new(
        source: impl Into<String>,
        target: impl Into<String>,
        source_lang: impl Into<String>,
        target_lang: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            source: source.into(),
            target: target.into(),
            source_lang: source_lang.into(),
            target_lang: target_lang.into(),
            created_at: now,
            modified_at: now,
            metadata: RecordMetadata::default(),
        }
    }

    pub fn with_metadata(mut self, metadata: RecordMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RecordMetadata {
    /// Who created/last edited this record
    pub creator: Option<String>,
    /// Client name (for filtering)
    pub client: Option<String>,
    /// Domain/subject matter (legal, medical, technical, ...)
    pub domain: Option<String>,
    /// 0–100, matches Felix's original reliability field
    pub reliability: u8,
    /// Has this translation been reviewed and approved?
    pub validated: bool,
}

/// A TM match returned by the matcher.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmMatch {
    pub record: Record,
    /// 0.0 (no match) to 1.0 (exact match)
    pub score: f32,
    pub match_type: MatchType,
}

impl TmMatch {
    pub fn score_percent(&self) -> u8 {
        (self.score * 100.0).round() as u8
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MatchType {
    /// 100% character-level identical after normalisation
    Exact,
    /// Between threshold and 100%
    Fuzzy,
    /// Same neighbours also matched (context-aware TM match)
    ContextMatch,
}

/// Language pair helper
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct LangPair {
    pub source: String,
    pub target: String,
}

impl LangPair {
    pub fn new(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
        }
    }
}

/// Statistics returned after a TMX/XLIFF import
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ImportStats {
    pub imported: usize,
    pub skipped_duplicates: usize,
    pub skipped_errors: usize,
}
