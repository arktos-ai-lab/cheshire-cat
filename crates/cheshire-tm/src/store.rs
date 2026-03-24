use std::path::Path;

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use sqlx::{Row, SqlitePool, sqlite::SqlitePoolOptions};
use uuid::Uuid;

use crate::{
    error::Result,
    record::{ImportStats, LangPair, Record, RecordMetadata},
};

/// Persistent SQLite-backed translation memory store.
///
/// Uses runtime-checked queries (no compile-time DATABASE_URL needed),
/// making this library portable without build-time database setup.
pub struct TmStore {
    pub(crate) pool: SqlitePool,
}

impl TmStore {
    /// Open (or create) a TM database at the given path.
    pub async fn open(path: &Path) -> Result<Self> {
        let url = format!(
            "sqlite://{}?mode=rwc",
            path.to_string_lossy().replace('\\', "/")
        );
        let pool = SqlitePoolOptions::new()
            .max_connections(4)
            .connect(&url)
            .await?;

        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    /// Open an in-memory database (for tests).
    pub async fn in_memory() -> Result<Self> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    // ── Records ──────────────────────────────────────────────────────────

    pub async fn insert(&self, record: &Record) -> Result<Uuid> {
        let source_hash = hash_source(&record.source);
        let id = record.id.to_string();
        let created = record.created_at.to_rfc3339();
        let modified = record.modified_at.to_rfc3339();
        let reliability = record.metadata.reliability as i64;
        let validated = record.metadata.validated as i64;

        sqlx::query(
            r#"INSERT INTO records
               (id, source, target, source_lang, target_lang, source_hash,
                created_at, modified_at, creator, client, domain, reliability, validated)
               VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)"#,
        )
        .bind(&id)
        .bind(&record.source)
        .bind(&record.target)
        .bind(&record.source_lang)
        .bind(&record.target_lang)
        .bind(&source_hash)
        .bind(&created)
        .bind(&modified)
        .bind(&record.metadata.creator)
        .bind(&record.metadata.client)
        .bind(&record.metadata.domain)
        .bind(reliability)
        .bind(validated)
        .execute(&self.pool)
        .await?;

        Ok(record.id)
    }

    pub async fn update(&self, record: &Record) -> Result<()> {
        let source_hash = hash_source(&record.source);
        let now = Utc::now().to_rfc3339();
        let id = record.id.to_string();
        let reliability = record.metadata.reliability as i64;
        let validated = record.metadata.validated as i64;

        sqlx::query(
            r#"UPDATE records SET
               source=?, target=?, source_hash=?, modified_at=?,
               creator=?, client=?, domain=?, reliability=?, validated=?
               WHERE id=?"#,
        )
        .bind(&record.source)
        .bind(&record.target)
        .bind(&source_hash)
        .bind(&now)
        .bind(&record.metadata.creator)
        .bind(&record.metadata.client)
        .bind(&record.metadata.domain)
        .bind(reliability)
        .bind(validated)
        .bind(&id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM records WHERE id=?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<Record>> {
        let row = sqlx::query("SELECT * FROM records WHERE id=?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| row_to_record(&r)))
    }

    /// Fast exact-match lookup using a pre-computed source hash.
    pub async fn exact_matches(
        &self,
        source: &str,
        lang_pair: &LangPair,
    ) -> Result<Vec<Record>> {
        let hash = hash_source(source);
        let rows = sqlx::query(
            "SELECT * FROM records WHERE source_hash=? AND source_lang=? AND target_lang=?",
        )
        .bind(&hash)
        .bind(&lang_pair.source)
        .bind(&lang_pair.target)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(row_to_record).collect())
    }

    /// Load all records for a language pair into memory (seeds the Matcher).
    pub async fn load_for_matching(&self, lang_pair: &LangPair) -> Result<Vec<Record>> {
        let rows = sqlx::query(
            "SELECT * FROM records WHERE source_lang=? AND target_lang=?",
        )
        .bind(&lang_pair.source)
        .bind(&lang_pair.target)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(row_to_record).collect())
    }

    /// Total record count.
    pub async fn count(&self) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as n FROM records")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.get::<i64, _>("n"))
    }

    // ── TMX ──────────────────────────────────────────────────────────────

    pub async fn import_tmx(&self, path: &Path) -> Result<ImportStats> {
        crate::tmx::import(path, self).await
    }

    pub async fn import_xliff(&self, path: &Path) -> Result<ImportStats> {
        crate::xliff::import(path, self).await
    }

    /// Return a [`GlossaryStore`] that shares this database connection pool.
    pub fn glossary_store(&self) -> crate::glossary::GlossaryStore {
        crate::glossary::GlossaryStore::new(self.pool.clone())
    }

    /// List records for a language pair with limit/offset pagination.
    pub async fn list_records(
        &self,
        lang_pair: &LangPair,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Record>> {
        let rows = sqlx::query(
            "SELECT * FROM records WHERE source_lang=? AND target_lang=? \
             ORDER BY modified_at DESC LIMIT ? OFFSET ?",
        )
        .bind(&lang_pair.source)
        .bind(&lang_pair.target)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(row_to_record).collect())
    }

    pub async fn export_tmx(&self, path: &Path, lang_pair: Option<&LangPair>) -> Result<()> {
        let records = if let Some(lp) = lang_pair {
            self.load_for_matching(lp).await?
        } else {
            let rows = sqlx::query("SELECT * FROM records")
                .fetch_all(&self.pool)
                .await?;
            rows.iter().map(row_to_record).collect()
        };
        crate::tmx::export(path, &records).await
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// SHA-256 of NFC-normalised, lowercased source text.
pub fn hash_source(source: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    let normalised: String = source.nfc().collect::<String>().to_lowercase();
    let mut hasher = Sha256::new();
    hasher.update(normalised.as_bytes());
    hex::encode(hasher.finalize())
}

fn row_to_record(row: &sqlx::sqlite::SqliteRow) -> Record {
    let id_str: String = row.get("id");
    let created_str: String = row.get("created_at");
    let modified_str: String = row.get("modified_at");
    let reliability: i64 = row.get("reliability");
    let validated: i64 = row.get("validated");

    Record {
        id: id_str.parse().unwrap_or_else(|_| Uuid::new_v4()),
        source: row.get("source"),
        target: row.get("target"),
        source_lang: row.get("source_lang"),
        target_lang: row.get("target_lang"),
        created_at: DateTime::parse_from_rfc3339(&created_str)
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
        modified_at: DateTime::parse_from_rfc3339(&modified_str)
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
        metadata: RecordMetadata {
            creator: row.get("creator"),
            client: row.get("client"),
            domain: row.get("domain"),
            reliability: reliability.clamp(0, 100) as u8,
            validated: validated != 0,
        },
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::Record;

    #[tokio::test]
    async fn insert_and_retrieve() {
        let store = TmStore::in_memory().await.unwrap();
        let record = Record::new("Hello world", "こんにちは世界", "en", "ja");
        let id = store.insert(&record).await.unwrap();

        let fetched = store.get(id).await.unwrap().expect("record not found");
        assert_eq!(fetched.source, "Hello world");
        assert_eq!(fetched.target, "こんにちは世界");
    }

    #[tokio::test]
    async fn count_increments() {
        let store = TmStore::in_memory().await.unwrap();
        assert_eq!(store.count().await.unwrap(), 0);
        store.insert(&Record::new("a", "b", "en", "ja")).await.unwrap();
        store.insert(&Record::new("c", "d", "en", "ja")).await.unwrap();
        assert_eq!(store.count().await.unwrap(), 2);
    }

    #[tokio::test]
    async fn delete_removes_record() {
        let store = TmStore::in_memory().await.unwrap();
        let record = Record::new("Hello", "こんにちは", "en", "ja");
        let id = store.insert(&record).await.unwrap();
        store.delete(id).await.unwrap();
        assert!(store.get(id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn exact_match_by_hash() {
        let store = TmStore::in_memory().await.unwrap();
        store
            .insert(&Record::new("Sign the document", "書類に署名する", "en", "ja"))
            .await
            .unwrap();

        let lp = LangPair::new("en", "ja");
        let hits = store.exact_matches("Sign the document", &lp).await.unwrap();
        assert_eq!(hits.len(), 1);

        // Case-insensitive (normalised before hashing)
        let hits2 = store.exact_matches("SIGN THE DOCUMENT", &lp).await.unwrap();
        assert_eq!(hits2.len(), 1);
    }

    #[tokio::test]
    async fn load_for_matching_filters_by_lang() {
        let store = TmStore::in_memory().await.unwrap();
        store.insert(&Record::new("Hello", "こんにちは", "en", "ja")).await.unwrap();
        store.insert(&Record::new("Hello", "Hallo", "en", "de")).await.unwrap();

        let lp_ja = LangPair::new("en", "ja");
        let ja = store.load_for_matching(&lp_ja).await.unwrap();
        assert_eq!(ja.len(), 1);
        assert_eq!(ja[0].target_lang, "ja");
    }

    #[tokio::test]
    async fn update_modifies_target() {
        let store = TmStore::in_memory().await.unwrap();
        let mut record = Record::new("Hello", "こんにちは", "en", "ja");
        store.insert(&record).await.unwrap();

        record.target = "やあ".to_string();
        store.update(&record).await.unwrap();

        let fetched = store.get(record.id).await.unwrap().unwrap();
        assert_eq!(fetched.target, "やあ");
    }
}
