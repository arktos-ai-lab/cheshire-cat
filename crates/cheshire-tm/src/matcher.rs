use std::collections::HashSet;
use std::sync::Arc;

use rayon::prelude::*;
use unicode_normalization::UnicodeNormalization;

use crate::record::{MatchType, Record, TmMatch};

/// In-memory fuzzy matcher.
///
/// Load once with [`Matcher::new`], then call [`Matcher::search`] for every
/// query — all search work is done in RAM, so queries return in milliseconds
/// even on large TMs.
///
/// Uses the same 3-pass algorithm as the original Felix C++ implementation:
/// 1. Bag-of-words pre-filter (eliminate obvious non-matches quickly)
/// 2. Levenshtein edit distance on remaining candidates (parallelised via rayon)
/// 3. Threshold filter and sort
pub struct Matcher {
    candidates: Arc<Vec<Record>>,
}

impl Matcher {
    pub fn new(records: Vec<Record>) -> Self {
        Self {
            candidates: Arc::new(records),
        }
    }

    pub fn record_count(&self) -> usize {
        self.candidates.len()
    }

    /// Add a newly-confirmed record to the in-memory index without reloading.
    pub fn add(&mut self, record: Record) {
        Arc::make_mut(&mut self.candidates).push(record);
    }

    /// Search for the best matches for `query`.
    ///
    /// - `threshold`: minimum score (0.0–1.0) to include in results
    /// - `max_results`: cap on number of results returned
    pub fn search(&self, query: &str, threshold: f32, max_results: usize) -> Vec<TmMatch> {
        let query_norm = normalise(query);
        let query_chars: Vec<char> = query_norm.chars().collect();
        let query_len = query_chars.len();

        if query_len == 0 {
            return Vec::new();
        }

        let query_words: HashSet<&str> = query_norm.split_whitespace().collect();

        let mut matches: Vec<TmMatch> = self
            .candidates
            .par_iter()
            // ── Pass 1: bag-of-words pre-filter ──────────────────────────
            // Quickly discard records that share too few words with the query.
            // This eliminates most candidates before the expensive edit distance.
            .filter(|record| {
                let rec_norm = normalise(&record.source);
                let rec_words: HashSet<&str> = rec_norm.split_whitespace().collect();

                let shared = query_words.intersection(&rec_words).count();
                let min_words = query_words.len().min(rec_words.len());

                // Allow records where at least (threshold - 0.15) of words match.
                // The 0.15 margin accounts for word order differences.
                if min_words == 0 {
                    return query_words.is_empty();
                }
                (shared as f32 / min_words as f32) >= (threshold - 0.15).max(0.0)
            })
            // ── Pass 2: edit distance ─────────────────────────────────────
            .map(|record| {
                let rec_norm = normalise(&record.source);
                let rec_chars: Vec<char> = rec_norm.chars().collect();
                let score = levenshtein_score(&query_chars, &rec_chars);
                (record, score)
            })
            // ── Pass 3: threshold filter ──────────────────────────────────
            .filter(|(_, score)| *score >= threshold)
            .map(|(record, score)| {
                let match_type = if score >= 0.9999 {
                    MatchType::Exact
                } else {
                    MatchType::Fuzzy
                };
                TmMatch {
                    record: record.clone(),
                    score,
                    match_type,
                }
            })
            .collect();

        // Sort by score descending, stable (preserves insertion order on tie)
        matches.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        matches.truncate(max_results);
        matches
    }
}

// ── Text normalisation ────────────────────────────────────────────────────────

/// Normalise text before comparison:
/// - Unicode NFC normalisation
/// - Lowercase
/// - Collapse whitespace
///
/// This matches Felix's original normalisation approach.
pub fn normalise(text: &str) -> String {
    let nfc: String = text.nfc().collect();
    let lower = nfc.to_lowercase();
    // Collapse all whitespace sequences to a single space
    lower
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

// ── Levenshtein edit distance ─────────────────────────────────────────────────

/// Compute a similarity score (0.0–1.0) between two character sequences using
/// the Levenshtein edit distance, normalised by the length of the longer string.
///
/// score = 1.0 - (edit_distance / max_len)
fn levenshtein_score(a: &[char], b: &[char]) -> f32 {
    let len_a = a.len();
    let len_b = b.len();

    if len_a == 0 && len_b == 0 {
        return 1.0;
    }
    let max_len = len_a.max(len_b);
    if max_len == 0 {
        return 1.0;
    }

    // Early exit: if length difference alone makes a match impossible, skip
    let len_diff = (len_a as isize - len_b as isize).unsigned_abs();
    if len_diff >= max_len {
        return 0.0;
    }

    let dist = edit_distance(a, b);
    1.0 - (dist as f32 / max_len as f32)
}

/// Standard Levenshtein distance (Wagner-Fischer algorithm).
/// Operates on char slices for correct Unicode handling.
fn edit_distance(a: &[char], b: &[char]) -> usize {
    let len_a = a.len();
    let len_b = b.len();

    // Use two rows to keep memory usage O(min(m,n))
    let (a, b, len_a, len_b) = if len_a < len_b {
        (a, b, len_a, len_b)
    } else {
        (b, a, len_b, len_a)
    };

    let mut prev: Vec<usize> = (0..=len_a).collect();
    let mut curr = vec![0usize; len_a + 1];

    for j in 1..=len_b {
        curr[0] = j;
        for i in 1..=len_a {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[i] = (prev[i] + 1)        // deletion
                .min(curr[i - 1] + 1)      // insertion
                .min(prev[i - 1] + cost);  // substitution
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[len_a]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::Record;

    fn make_record(source: &str, target: &str) -> Record {
        Record::new(source, target, "en", "ja")
    }

    #[test]
    fn exact_match_scores_one() {
        let records = vec![make_record("Hello world", "こんにちは世界")];
        let matcher = Matcher::new(records);
        let results = matcher.search("Hello world", 0.5, 5);
        assert_eq!(results.len(), 1);
        assert!((results[0].score - 1.0).abs() < 1e-6);
        assert_eq!(results[0].match_type, MatchType::Exact);
    }

    #[test]
    fn fuzzy_match_in_range() {
        let records = vec![make_record(
            "Please sign the document before filing",
            "提出前に書類に署名してください",
        )];
        let matcher = Matcher::new(records);
        // One word changed
        let results = matcher.search("Please sign the document before submission", 0.5, 5);
        assert_eq!(results.len(), 1);
        assert!(results[0].score > 0.5 && results[0].score < 1.0);
        assert_eq!(results[0].match_type, MatchType::Fuzzy);
    }

    #[test]
    fn below_threshold_excluded() {
        let records = vec![make_record("The cat sat on the mat", "猫がマットの上に座った")];
        let matcher = Matcher::new(records);
        let results = matcher.search("Completely different sentence here", 0.6, 5);
        assert!(results.is_empty());
    }

    #[test]
    fn results_sorted_by_score_descending() {
        let records = vec![
            make_record("Hello world", "こんにちは世界"),
            make_record("Hello there world", "そこにいるこんにちは世界"),
            make_record("Hello beautiful world", "美しい世界よこんにちは"),
        ];
        let matcher = Matcher::new(records);
        let results = matcher.search("Hello world", 0.5, 5);
        for window in results.windows(2) {
            assert!(window[0].score >= window[1].score);
        }
    }

    #[test]
    fn empty_query_returns_empty() {
        let records = vec![make_record("Hello", "こんにちは")];
        let matcher = Matcher::new(records);
        assert!(matcher.search("", 0.5, 5).is_empty());
    }

    #[test]
    fn empty_tm_returns_empty() {
        let matcher = Matcher::new(vec![]);
        assert!(matcher.search("Hello world", 0.5, 5).is_empty());
    }

    #[test]
    fn max_results_respected() {
        let records: Vec<Record> = (0..20)
            .map(|i| make_record(&format!("Hello world {}", i), "こんにちは"))
            .collect();
        let matcher = Matcher::new(records);
        let results = matcher.search("Hello world", 0.5, 3);
        assert!(results.len() <= 3);
    }

    #[test]
    fn case_insensitive_matching() {
        let records = vec![make_record("hello world", "こんにちは世界")];
        let matcher = Matcher::new(records);
        let results = matcher.search("HELLO WORLD", 0.9, 5);
        assert_eq!(results.len(), 1);
        assert!((results[0].score - 1.0).abs() < 1e-6);
    }

    #[test]
    fn normalise_collapses_whitespace() {
        assert_eq!(normalise("hello   world"), "hello world");
        assert_eq!(normalise("  leading"), "leading");
        assert_eq!(normalise("trailing  "), "trailing");
    }

    #[test]
    fn edit_distance_correct() {
        let a: Vec<char> = "kitten".chars().collect();
        let b: Vec<char> = "sitting".chars().collect();
        assert_eq!(edit_distance(&a, &b), 3);

        let same: Vec<char> = "hello".chars().collect();
        assert_eq!(edit_distance(&same, &same), 0);
    }
}
