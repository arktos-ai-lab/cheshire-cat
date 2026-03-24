//! Integration tests for cheshire-tm covering real translation workflow scenarios.
//!
//! These tests work against an in-memory SQLite database and exercise the full
//! stack: TmStore → Matcher → GlossaryStore.

use cheshire_tm::{
    record::{LangPair, Record, RecordMetadata},
    GlossaryStore, GlossaryTerm, Matcher, TmMatch, TmStore,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn make_store() -> TmStore {
    TmStore::in_memory().await.unwrap()
}

fn en_ja(source: &str, target: &str) -> Record {
    Record::new(source, target, "en", "ja")
}

fn en_de(source: &str, target: &str) -> Record {
    Record::new(source, target, "en", "de")
}

// ── TmStore ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn store_insert_and_retrieve_roundtrip() {
    let tm = make_store().await;
    let record = en_ja("Please sign the document.", "書類に署名してください。");
    let id = tm.insert(&record).await.unwrap();

    let fetched = tm.get(id).await.unwrap().expect("record not found");
    assert_eq!(fetched.source, record.source);
    assert_eq!(fetched.target, record.target);
    assert_eq!(fetched.source_lang, "en");
    assert_eq!(fetched.target_lang, "ja");
}

#[tokio::test]
async fn store_unicode_source_target_roundtrip() {
    let tm = make_store().await;
    // Full-width and CJK characters
    let record = en_ja(
        "Ｈｅｌｌｏ　ｗｏｒｌｄ",
        "日本語テキスト：漢字ひらがなカタカナ",
    );
    let id = tm.insert(&record).await.unwrap();
    let fetched = tm.get(id).await.unwrap().unwrap();
    assert_eq!(fetched.source, "Ｈｅｌｌｏ　ｗｏｒｌｄ");
    assert_eq!(fetched.target, "日本語テキスト：漢字ひらがなカタカナ");
}

#[tokio::test]
async fn store_metadata_preserved() {
    let tm = make_store().await;
    let mut record = en_ja("Contract clause 3.1", "契約条項 3.1");
    record.metadata = RecordMetadata {
        creator: Some("Alice".into()),
        client: Some("ACME Corp".into()),
        domain: Some("Legal".into()),
        reliability: 95,
        validated: true,
    };
    let id = tm.insert(&record).await.unwrap();
    let fetched = tm.get(id).await.unwrap().unwrap();
    assert_eq!(fetched.metadata.creator.as_deref(), Some("Alice"));
    assert_eq!(fetched.metadata.client.as_deref(), Some("ACME Corp"));
    assert_eq!(fetched.metadata.domain.as_deref(), Some("Legal"));
    assert_eq!(fetched.metadata.reliability, 95);
    assert!(fetched.metadata.validated);
}

#[tokio::test]
async fn store_reliability_stored_accurately() {
    let tm = make_store().await;
    for rel in [0u8, 50, 75, 100] {
        let mut record = en_ja(&format!("Test {rel}"), "テスト");
        record.metadata.reliability = rel;
        let id = tm.insert(&record).await.unwrap();
        let fetched = tm.get(id).await.unwrap().unwrap();
        assert_eq!(fetched.metadata.reliability, rel);
    }
}

#[tokio::test]
async fn store_update_changes_target_and_increments_modified() {
    let tm = make_store().await;
    let mut record = en_ja("Hello", "こんにちは");
    let original_modified = record.modified_at;
    tm.insert(&record).await.unwrap();

    record.target = "やあ".into();
    tm.update(&record).await.unwrap();

    let fetched = tm.get(record.id).await.unwrap().unwrap();
    assert_eq!(fetched.target, "やあ");
    // modified_at should be updated
    assert!(fetched.modified_at >= original_modified);
}

#[tokio::test]
async fn store_delete_removes_record() {
    let tm = make_store().await;
    let record = en_ja("Hello", "こんにちは");
    let id = tm.insert(&record).await.unwrap();

    tm.delete(id).await.unwrap();

    assert!(tm.get(id).await.unwrap().is_none());
    assert_eq!(tm.count().await.unwrap(), 0);
}

#[tokio::test]
async fn store_exact_match_case_insensitive() {
    let tm = make_store().await;
    tm.insert(&en_ja("Sign the document", "書類に署名する")).await.unwrap();

    let lp = LangPair::new("en", "ja");
    // Original case
    let hits = tm.exact_matches("Sign the document", &lp).await.unwrap();
    assert_eq!(hits.len(), 1);

    // Upper case
    let hits2 = tm.exact_matches("SIGN THE DOCUMENT", &lp).await.unwrap();
    assert_eq!(hits2.len(), 1);

    // Mixed case
    let hits3 = tm.exact_matches("Sign The Document", &lp).await.unwrap();
    assert_eq!(hits3.len(), 1);
}

#[tokio::test]
async fn store_exact_match_wrong_lang_pair_no_hit() {
    let tm = make_store().await;
    tm.insert(&en_ja("Hello", "こんにちは")).await.unwrap();

    let lp = LangPair::new("en", "de");
    let hits = tm.exact_matches("Hello", &lp).await.unwrap();
    assert!(hits.is_empty());
}

#[tokio::test]
async fn store_list_records_pagination() {
    let tm = make_store().await;
    let lp = LangPair::new("en", "ja");

    for i in 0..10 {
        tm.insert(&en_ja(&format!("Sentence {i}"), "文")).await.unwrap();
    }

    let first_five = tm.list_records(&lp, 5, 0).await.unwrap();
    assert_eq!(first_five.len(), 5);

    let next_five = tm.list_records(&lp, 5, 5).await.unwrap();
    assert_eq!(next_five.len(), 5);

    // No overlap
    let ids_first: std::collections::HashSet<uuid::Uuid> =
        first_five.iter().map(|r| r.id).collect();
    let ids_next: std::collections::HashSet<uuid::Uuid> =
        next_five.iter().map(|r| r.id).collect();
    assert!(ids_first.is_disjoint(&ids_next));
}

#[tokio::test]
async fn store_list_records_filters_by_lang_pair() {
    let tm = make_store().await;
    tm.insert(&en_ja("Hello", "こんにちは")).await.unwrap();
    tm.insert(&en_de("Hello", "Hallo")).await.unwrap();

    let en_ja_records = tm.list_records(&LangPair::new("en", "ja"), 100, 0).await.unwrap();
    assert_eq!(en_ja_records.len(), 1);
    assert_eq!(en_ja_records[0].target_lang, "ja");
}

#[tokio::test]
async fn store_glossary_store_shares_database() {
    // Verify that glossary_store() works against the same DB.
    let tm = make_store().await;
    let glossary = tm.glossary_store();

    let term = GlossaryTerm::new("submission", "提出", "en", "ja");
    glossary.insert(&term).await.unwrap();

    let count = glossary.count().await.unwrap();
    assert_eq!(count, 1);

    let hits = glossary
        .lookup_in("Please sign before submission", "en", "ja")
        .await
        .unwrap();
    assert_eq!(hits.len(), 1);
}

#[tokio::test]
async fn store_load_for_matching_multiple_records() {
    let tm = make_store().await;
    for i in 0..5 {
        tm.insert(&en_ja(&format!("Record {i}"), "記録")).await.unwrap();
    }
    let lp = LangPair::new("en", "ja");
    let all = tm.load_for_matching(&lp).await.unwrap();
    assert_eq!(all.len(), 5);
    assert!(all.iter().all(|r| r.source_lang == "en" && r.target_lang == "ja"));
}

// ── Matcher ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn matcher_exact_match_roundtrip_through_store() {
    let tm = make_store().await;
    tm.insert(&en_ja("Please sign the document.", "書類に署名してください。"))
        .await
        .unwrap();

    let lp = LangPair::new("en", "ja");
    let records = tm.load_for_matching(&lp).await.unwrap();
    let matcher = Matcher::new(records);

    let results = matcher.search("Please sign the document.", 0.9, 5);
    assert_eq!(results.len(), 1);
    assert!((results[0].score - 1.0).abs() < 1e-6);
}

#[test]
fn matcher_unicode_nfc_normalization() {
    // Precomposed "é" (U+00E9) vs decomposed "e" + combining accent (U+0065 U+0301)
    // After NFC normalisation both should be equal.
    let record_precomposed = Record::new("café", "カフェ", "en", "ja");
    let matcher = Matcher::new(vec![record_precomposed]);

    let decomposed = "cafe\u{0301}"; // e + combining accent
    let results = matcher.search(decomposed, 0.9, 5);
    assert_eq!(results.len(), 1, "NFC-normalized forms should match");
}

#[test]
fn matcher_single_word_exact() {
    let matcher = Matcher::new(vec![Record::new("contract", "契約", "en", "ja")]);
    let results = matcher.search("contract", 0.9, 5);
    assert_eq!(results.len(), 1);
    assert!((results[0].score - 1.0).abs() < 1e-6);
}

#[test]
fn matcher_one_word_change_fuzzy() {
    let matcher = Matcher::new(vec![Record::new(
        "Please sign the document",
        "書類に署名してください",
        "en",
        "ja",
    )]);
    // "agreement" replaces "document"
    let results = matcher.search("Please sign the agreement", 0.6, 5);
    assert_eq!(results.len(), 1);
    assert!(results[0].score < 1.0 && results[0].score >= 0.6);
}

#[test]
fn matcher_add_record_is_immediately_searchable() {
    let mut matcher = Matcher::new(vec![]);
    assert!(matcher.search("Hello world", 0.9, 5).is_empty());

    matcher.add(Record::new("Hello world", "こんにちは世界", "en", "ja"));
    let results = matcher.search("Hello world", 0.9, 5);
    assert_eq!(results.len(), 1);
}

#[test]
fn matcher_record_count_after_add() {
    let mut matcher = Matcher::new(vec![]);
    assert_eq!(matcher.record_count(), 0);

    for i in 0..5 {
        matcher.add(Record::new(format!("Record {i}"), "記録", "en", "ja"));
    }
    assert_eq!(matcher.record_count(), 5);
}

#[test]
fn matcher_score_percent_rounds_correctly() {
    let tm_match = TmMatch {
        record: Record::new("a", "b", "en", "ja"),
        score: 0.756,
        match_type: cheshire_tm::MatchType::Fuzzy,
    };
    assert_eq!(tm_match.score_percent(), 76);
}

#[test]
fn matcher_high_threshold_filters_fuzzy() {
    let matcher = Matcher::new(vec![Record::new(
        "Sign the document please",
        "書類に署名してください",
        "en",
        "ja",
    )]);
    // With 99% threshold, a fuzzy match should not appear.
    let results = matcher.search("Sign the contract please", 0.99, 5);
    assert!(results.is_empty());
}

#[test]
fn matcher_very_long_source_no_panic() {
    // 10,000 character source — should not stack overflow or panic.
    let long = "word ".repeat(2000);
    let matcher = Matcher::new(vec![Record::new(long.trim(), "テスト", "en", "ja")]);
    let results = matcher.search(long.trim(), 0.9, 5);
    assert_eq!(results.len(), 1);
}

#[test]
fn matcher_multiple_identical_candidates_all_returned_up_to_max() {
    let records: Vec<Record> = (0..10)
        .map(|_| Record::new("identical source", "同じソース", "en", "ja"))
        .collect();
    let matcher = Matcher::new(records);
    let results = matcher.search("identical source", 0.9, 5);
    assert_eq!(results.len(), 5); // capped at max
}

// ── GlossaryStore ─────────────────────────────────────────────────────────────

async fn glossary_store() -> GlossaryStore {
    let tm = TmStore::in_memory().await.unwrap();
    tm.glossary_store()
}

#[tokio::test]
async fn glossary_forbidden_excluded_from_lookup() {
    let gs = glossary_store().await;
    let mut term = GlossaryTerm::new("submission", "提出", "en", "ja");
    term.forbidden = true;
    gs.insert(&term).await.unwrap();

    let hits = gs
        .lookup_in("Please sign before submission", "en", "ja")
        .await
        .unwrap();
    assert!(hits.is_empty());
}

#[tokio::test]
async fn glossary_multiple_terms_found_in_order() {
    let gs = glossary_store().await;
    gs.insert(&GlossaryTerm::new("sign", "署名する", "en", "ja")).await.unwrap();
    gs.insert(&GlossaryTerm::new("document", "書類", "en", "ja")).await.unwrap();

    let hits = gs
        .lookup_in("Please sign the document", "en", "ja")
        .await
        .unwrap();
    assert_eq!(hits.len(), 2);
    assert!(hits[0].offset < hits[1].offset); // ordered by position
}

#[tokio::test]
async fn glossary_case_insensitive_lookup() {
    let gs = glossary_store().await;
    gs.insert(&GlossaryTerm::new("contract", "契約", "en", "ja")).await.unwrap();

    let hits = gs
        .lookup_in("The CONTRACT must be signed", "en", "ja")
        .await
        .unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].term.target_term, "契約");
}

#[tokio::test]
async fn glossary_domain_stored_and_returned() {
    let gs = glossary_store().await;
    let mut term = GlossaryTerm::new("jurisdiction", "管轄権", "en", "ja");
    term.domain = Some("Legal".into());
    gs.insert(&term).await.unwrap();

    let hits = gs
        .lookup_in("The court has jurisdiction", "en", "ja")
        .await
        .unwrap();
    assert_eq!(hits[0].term.domain.as_deref(), Some("Legal"));
}

#[tokio::test]
async fn glossary_delete_removes_term() {
    let gs = glossary_store().await;
    let term = GlossaryTerm::new("contract", "契約", "en", "ja");
    let id = gs.insert(&term).await.unwrap();
    assert_eq!(gs.count().await.unwrap(), 1);

    gs.delete(id).await.unwrap();
    assert_eq!(gs.count().await.unwrap(), 0);

    let hits = gs.lookup_in("Signing the contract", "en", "ja").await.unwrap();
    assert!(hits.is_empty());
}

#[tokio::test]
async fn glossary_different_lang_pairs_isolated() {
    let gs = glossary_store().await;
    gs.insert(&GlossaryTerm::new("contract", "契約", "en", "ja")).await.unwrap();
    gs.insert(&GlossaryTerm::new("contract", "Vertrag", "en", "de")).await.unwrap();

    let ja_hits = gs.lookup_in("The contract", "en", "ja").await.unwrap();
    assert_eq!(ja_hits.len(), 1);
    assert_eq!(ja_hits[0].term.target_term, "契約");

    let de_hits = gs.lookup_in("The contract", "en", "de").await.unwrap();
    assert_eq!(de_hits.len(), 1);
    assert_eq!(de_hits[0].term.target_term, "Vertrag");
}

#[tokio::test]
async fn glossary_term_not_in_text_returns_no_hit() {
    let gs = glossary_store().await;
    gs.insert(&GlossaryTerm::new("jurisdiction", "管轄権", "en", "ja")).await.unwrap();

    let hits = gs
        .lookup_in("Please sign the contract", "en", "ja")
        .await
        .unwrap();
    assert!(hits.is_empty());
}

// ── TMX import/export ─────────────────────────────────────────────────────────

#[tokio::test]
async fn tmx_roundtrip_preserves_source_and_target() {
    use tempfile::NamedTempFile;

    let tm = make_store().await;
    let records = vec![
        en_ja("Please sign the document.", "書類に署名してください。"),
        en_ja("The contract is valid.", "契約は有効です。"),
    ];
    for r in &records {
        tm.insert(r).await.unwrap();
    }

    // Export
    let tmp = NamedTempFile::new().unwrap();
    tm.export_tmx(tmp.path(), Some(&LangPair::new("en", "ja")))
        .await
        .unwrap();

    // Import into a fresh store
    let tm2 = make_store().await;
    let stats = tm2.import_tmx(tmp.path()).await.unwrap();
    assert_eq!(stats.imported, 2);
    assert_eq!(stats.skipped_errors, 0);

    let lp = LangPair::new("en", "ja");
    let imported = tm2.load_for_matching(&lp).await.unwrap();
    assert_eq!(imported.len(), 2);

    let sources: std::collections::HashSet<&str> =
        imported.iter().map(|r| r.source.as_str()).collect();
    assert!(sources.contains("Please sign the document."));
    assert!(sources.contains("The contract is valid."));
}

#[tokio::test]
async fn tmx_import_skips_duplicates() {
    use tempfile::NamedTempFile;

    let tm = make_store().await;
    tm.insert(&en_ja("Hello", "こんにちは")).await.unwrap();

    let tmp = NamedTempFile::new().unwrap();
    tm.export_tmx(tmp.path(), None).await.unwrap();

    // Import into the same store — should skip the duplicate
    let stats = tm.import_tmx(tmp.path()).await.unwrap();
    assert_eq!(stats.imported, 0);
    assert_eq!(stats.skipped_duplicates, 1);
    assert_eq!(tm.count().await.unwrap(), 1);
}

#[tokio::test]
async fn tmx_export_only_requested_lang_pair() {
    use tempfile::NamedTempFile;

    let tm = make_store().await;
    tm.insert(&en_ja("Hello", "こんにちは")).await.unwrap();
    tm.insert(&en_de("Hello", "Hallo")).await.unwrap();

    let tmp = NamedTempFile::new().unwrap();
    tm.export_tmx(tmp.path(), Some(&LangPair::new("en", "ja")))
        .await
        .unwrap();

    let tm2 = make_store().await;
    let stats = tm2.import_tmx(tmp.path()).await.unwrap();
    assert_eq!(stats.imported, 1);

    let ja = tm2
        .load_for_matching(&LangPair::new("en", "ja"))
        .await
        .unwrap();
    let de = tm2
        .load_for_matching(&LangPair::new("en", "de"))
        .await
        .unwrap();
    assert_eq!(ja.len(), 1);
    assert!(de.is_empty());
}

// ── End-to-end workflow ───────────────────────────────────────────────────────

#[tokio::test]
async fn end_to_end_confirm_segment_updates_matcher() {
    // Simulate a user confirming a segment: it appears in the DB and in
    // the live in-memory matcher without needing a restart.
    let tm = make_store().await;
    let lp = LangPair::new("en", "ja");
    let initial = tm.load_for_matching(&lp).await.unwrap();
    let mut matcher = Matcher::new(initial);

    // Initially empty
    assert!(matcher.search("Please sign the document", 0.9, 5).is_empty());

    // Confirm a segment
    let record = en_ja("Please sign the document.", "書類に署名してください。");
    tm.insert(&record).await.unwrap();
    matcher.add(record);

    // Now it appears immediately
    let results = matcher.search("Please sign the document.", 0.9, 5);
    assert_eq!(results.len(), 1);

    // And it persists in the DB
    assert_eq!(tm.count().await.unwrap(), 1);
}

#[tokio::test]
async fn end_to_end_glossary_and_tm_share_database() {
    let tm = make_store().await;
    let glossary = tm.glossary_store();

    // Insert a TM record
    tm.insert(&en_ja("Sign the contract.", "契約に署名する。"))
        .await
        .unwrap();

    // Insert a matching glossary term
    glossary
        .insert(&GlossaryTerm::new("contract", "契約", "en", "ja"))
        .await
        .unwrap();

    // Both are retrievable
    let tm_lp = LangPair::new("en", "ja");
    let tm_records = tm.load_for_matching(&tm_lp).await.unwrap();
    assert_eq!(tm_records.len(), 1);

    let gl_hits = glossary
        .lookup_in("Sign the contract.", "en", "ja")
        .await
        .unwrap();
    assert_eq!(gl_hits.len(), 1);
    assert_eq!(gl_hits[0].term.target_term, "契約");
}
