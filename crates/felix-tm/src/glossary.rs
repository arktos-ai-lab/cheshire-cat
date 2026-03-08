use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryTerm {
    pub id: Uuid,
    pub source_term: String,
    pub target_term: String,
    pub source_lang: String,
    pub target_lang: String,
    pub domain: Option<String>,
    pub note: Option<String>,
    pub forbidden: bool,
}

impl GlossaryTerm {
    pub fn new(
        source_term: impl Into<String>,
        target_term: impl Into<String>,
        source_lang: impl Into<String>,
        target_lang: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            source_term: source_term.into(),
            target_term: target_term.into(),
            source_lang: source_lang.into(),
            target_lang: target_lang.into(),
            domain: None,
            note: None,
            forbidden: false,
        }
    }
}

/// A glossary term found inside a source segment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryHit {
    pub term: GlossaryTerm,
    /// Byte offset in the source string where the term was found.
    pub offset: usize,
}

pub struct GlossaryStore {
    pool: SqlitePool,
}

impl GlossaryStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, term: &GlossaryTerm) -> Result<Uuid> {
        let id = term.id.to_string();
        let forbidden = term.forbidden as i64;
        sqlx::query(
            r#"INSERT INTO glossary_terms
               (id, source_term, target_term, source_lang, target_lang, domain, note, forbidden)
               VALUES (?,?,?,?,?,?,?,?)"#,
        )
        .bind(&id)
        .bind(&term.source_term)
        .bind(&term.target_term)
        .bind(&term.source_lang)
        .bind(&term.target_lang)
        .bind(&term.domain)
        .bind(&term.note)
        .bind(forbidden)
        .execute(&self.pool)
        .await?;
        Ok(term.id)
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM glossary_terms WHERE id=?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Find all non-forbidden glossary terms whose source_term appears in `text`.
    /// Case-insensitive. Results ordered by position in text.
    pub async fn lookup_in(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<GlossaryHit>> {
        let rows = sqlx::query(
            r#"SELECT id, source_term, target_term, source_lang, target_lang,
                      domain, note, forbidden
               FROM glossary_terms
               WHERE source_lang=? AND target_lang=? AND forbidden=0"#,
        )
        .bind(source_lang)
        .bind(target_lang)
        .fetch_all(&self.pool)
        .await?;

        let text_lower = text.to_lowercase();
        let mut hits: Vec<GlossaryHit> = Vec::new();

        for row in &rows {
            let source_term: String = row.get("source_term");
            let term_lower = source_term.to_lowercase();
            if let Some(offset) = text_lower.find(&term_lower) {
                let id_str: String = row.get("id");
                let forbidden: i64 = row.get("forbidden");
                hits.push(GlossaryHit {
                    term: GlossaryTerm {
                        id: id_str.parse().unwrap_or_else(|_| Uuid::new_v4()),
                        source_term,
                        target_term: row.get("target_term"),
                        source_lang: row.get("source_lang"),
                        target_lang: row.get("target_lang"),
                        domain: row.get("domain"),
                        note: row.get("note"),
                        forbidden: forbidden != 0,
                    },
                    offset,
                });
            }
        }

        hits.sort_by_key(|h| h.offset);
        Ok(hits)
    }

    pub async fn count(&self) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as n FROM glossary_terms")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.get::<i64, _>("n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn make_store() -> GlossaryStore {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        GlossaryStore::new(pool)
    }

    #[tokio::test]
    async fn insert_and_lookup_hit() {
        let gs = make_store().await;
        let term = GlossaryTerm::new("submission", "提出", "en", "ja");
        gs.insert(&term).await.unwrap();

        let hits = gs
            .lookup_in("Please sign before submission", "en", "ja")
            .await
            .unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].term.target_term, "提出");
    }

    #[tokio::test]
    async fn no_hit_when_absent() {
        let gs = make_store().await;
        gs.insert(&GlossaryTerm::new("contract", "契約", "en", "ja"))
            .await
            .unwrap();
        let hits = gs
            .lookup_in("Please sign before submission", "en", "ja")
            .await
            .unwrap();
        assert!(hits.is_empty());
    }

    #[tokio::test]
    async fn multiple_hits_ordered_by_position() {
        let gs = make_store().await;
        gs.insert(&GlossaryTerm::new("document", "書類", "en", "ja"))
            .await
            .unwrap();
        gs.insert(&GlossaryTerm::new("sign", "署名する", "en", "ja"))
            .await
            .unwrap();

        let hits = gs
            .lookup_in("Please sign the document", "en", "ja")
            .await
            .unwrap();
        assert_eq!(hits.len(), 2);
        assert!(hits[0].offset < hits[1].offset);
        assert_eq!(hits[0].term.source_term, "sign");
        assert_eq!(hits[1].term.source_term, "document");
    }

    #[tokio::test]
    async fn forbidden_terms_excluded() {
        let gs = make_store().await;
        let mut term = GlossaryTerm::new("submission", "提出", "en", "ja");
        term.forbidden = true;
        gs.insert(&term).await.unwrap();

        let hits = gs
            .lookup_in("Please sign before submission", "en", "ja")
            .await
            .unwrap();
        assert!(hits.is_empty());
    }
}
