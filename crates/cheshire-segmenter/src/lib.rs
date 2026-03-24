use regex::Regex;
use std::sync::OnceLock;

/// A single sentence extracted from a longer text unit.
#[derive(Debug, Clone)]
pub struct Segment {
    pub text: String,
    /// Zero-based position of this segment within the source unit.
    pub index: usize,
}

/// Tokens that are never sentence-final even when followed by a capital letter.
static ABBREVS: &[&str] = &[
    "mr", "mrs", "ms", "dr", "prof", "rev", "gen", "sgt", "pvt", "sr", "jr",
    "vs", "etc", "no", "vol", "fig", "dept", "approx", "cf", "al", "op",
];

static BREAK_RE: OnceLock<Regex> = OnceLock::new();

fn break_regex() -> &'static Regex {
    BREAK_RE.get_or_init(|| Regex::new(r"[.!?]+\s+").unwrap())
}

/// Split `text` into sentence-level [`Segment`]s.
///
/// Applies a simplified break rule: split on `.!?` followed by whitespace,
/// unless the token immediately before the punctuation is a known abbreviation.
pub fn segment(text: &str) -> Vec<Segment> {
    let re = break_regex();
    let mut segments: Vec<Segment> = Vec::new();
    let mut start = 0usize;
    let mut index = 0usize;

    for m in re.find_iter(text) {
        // The match spans "punctuation + whitespace"; find where the
        // punctuation ends so we can include it in the current segment.
        let punct_len = text[m.start()..m.end()]
            .chars()
            .take_while(|c| !c.is_whitespace())
            .map(|c| c.len_utf8())
            .sum::<usize>();
        let chunk_end = m.start() + punct_len;

        let chunk = text[start..chunk_end].trim();
        if chunk.is_empty() {
            start = m.end();
            continue;
        }

        // Reject break when the last word (stripped of trailing dots) is an
        // abbreviation, so "Dr. Smith" does not become two segments.
        let word_before = chunk
            .trim_end_matches(|c: char| ".!?".contains(c))
            .split_whitespace()
            .last()
            .unwrap_or("")
            .to_lowercase();

        if ABBREVS.iter().any(|a| word_before == *a) {
            continue;
        }

        segments.push(Segment { text: chunk.to_string(), index });
        index += 1;
        start = m.end();
    }

    // Any remaining text after the last break.
    let tail = text[start..].trim();
    if !tail.is_empty() {
        segments.push(Segment { text: tail.to_string(), index });
    }

    // If no splits occurred, return the whole text as a single segment.
    if segments.is_empty() && !text.trim().is_empty() {
        segments.push(Segment {
            text: text.trim().to_string(),
            index: 0,
        });
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Basic splitting ───────────────────────────────────────────────────────

    #[test]
    fn single_sentence_no_split() {
        let segs = segment("Hello world.");
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].text, "Hello world.");
    }

    #[test]
    fn two_simple_sentences() {
        let segs = segment("Hello world. How are you?");
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].text, "Hello world.");
        assert_eq!(segs[1].text, "How are you?");
    }

    #[test]
    fn three_sentences_sequential_indices() {
        let segs = segment("One. Two. Three.");
        assert_eq!(segs.len(), 3);
        let indices: Vec<usize> = segs.iter().map(|s| s.index).collect();
        assert_eq!(indices, vec![0, 1, 2]);
    }

    #[test]
    fn exclamation_and_question_split() {
        let segs = segment("Stop! Are you sure?");
        assert_eq!(segs.len(), 2);
    }

    #[test]
    fn multiple_punctuation_chars() {
        // "Wait... Really?" should split at the ellipsis.
        let segs = segment("Wait... Really?");
        assert_eq!(segs.len(), 2);
    }

    // ── Edge cases ────────────────────────────────────────────────────────────

    #[test]
    fn empty_text_returns_empty() {
        assert!(segment("").is_empty());
    }

    #[test]
    fn whitespace_only_returns_empty() {
        assert!(segment("   ").is_empty());
    }

    #[test]
    fn no_terminating_punctuation_is_single_segment() {
        let segs = segment("This sentence has no final punctuation");
        assert_eq!(segs.len(), 1);
    }

    #[test]
    fn leading_and_trailing_whitespace_trimmed() {
        let segs = segment("  Hello.  Goodbye.  ");
        assert_eq!(segs[0].text, "Hello.");
        assert_eq!(segs[1].text, "Goodbye.");
    }

    #[test]
    fn very_long_single_sentence_no_panic() {
        let long = "word ".repeat(5000) + "done.";
        let segs = segment(&long);
        assert_eq!(segs.len(), 1);
    }

    // ── Abbreviation handling ─────────────────────────────────────────────────

    #[test]
    fn abbreviation_dr_suppresses_break() {
        let segs = segment("Contact Dr. Smith for details.");
        assert_eq!(segs.len(), 1);
    }

    #[test]
    fn abbreviation_mr_suppresses_break() {
        let segs = segment("Please ask Mr. Johnson to sign.");
        assert_eq!(segs.len(), 1);
    }

    #[test]
    fn abbreviation_prof_suppresses_break() {
        let segs = segment("The findings by Prof. Nakamura are conclusive.");
        assert_eq!(segs.len(), 1);
    }

    #[test]
    fn abbreviation_etc_suppresses_break() {
        // "etc" is in the abbreviation list, so "etc. All" does NOT split.
        // Result: one segment containing the entire text.
        let segs = segment("Include contracts, receipts, etc. All must be signed.");
        assert_eq!(segs.len(), 1);
    }

    #[test]
    fn abbreviation_vs_suppresses_break() {
        let segs = segment("The case of Smith vs. Jones was settled.");
        assert_eq!(segs.len(), 1);
    }

    #[test]
    fn multiple_abbreviations_in_row() {
        let segs = segment("Contact Mr. Johnson or Dr. Smith for more info.");
        assert_eq!(segs.len(), 1);
    }

    // ── Unicode and multilingual ──────────────────────────────────────────────

    #[test]
    fn cjk_text_not_split_on_western_punctuation() {
        // Japanese text typically uses 。not ., so no split expected
        let segs = segment("書類に署名してください。次の文書をお読みください。");
        // Our current segmenter only splits on ASCII .!? followed by space.
        // CJK full-stop (。) is not in our pattern, so this is one segment.
        assert_eq!(segs.len(), 1);
    }

    #[test]
    fn unicode_text_with_western_split() {
        let segs = segment("Please sign. 書類に署名する。 Thank you.");
        // Should split on ". " after "Please sign" and after "ください。 "
        // Our regex only splits ASCII .!? + space, so:
        // "Please sign." + rest, then "Thank you." at end
        // "。 " does not match our pattern so it won't split there.
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].text, "Please sign.");
    }

    #[test]
    fn emoji_not_sentence_boundary() {
        let segs = segment("Great work! 🎉 Keep it up.");
        // "! " splits after "Great work"
        assert_eq!(segs.len(), 2);
    }

    // ── Segment index continuity ──────────────────────────────────────────────

    #[test]
    fn indices_are_always_sequential_from_zero() {
        let segs = segment("One. Two. Three. Four. Five.");
        for (i, seg) in segs.iter().enumerate() {
            assert_eq!(seg.index, i);
        }
    }

    #[test]
    fn single_segment_has_index_zero() {
        let segs = segment("Only one sentence here.");
        assert_eq!(segs[0].index, 0);
    }

    // ── Real CAT tool scenarios ───────────────────────────────────────────────

    #[test]
    fn legal_contract_clause() {
        let text = "The Contractor shall deliver the goods by the specified date. \
                    In case of delay, liquidated damages shall apply. \
                    The Contractor may not subcontract without prior written consent.";
        let segs = segment(text);
        assert_eq!(segs.len(), 3);
    }

    #[test]
    fn technical_manual_with_abbreviations() {
        let text = "See Fig. 3 for details. The voltage must not exceed 24V. \
                    Refer to Vol. 2 for advanced configuration.";
        let segs = segment(text);
        // "Fig." → suppressed (abbreviation); "details." → splits; "24V." → splits;
        // "Vol." → suppressed (abbreviation). Result: 3 segments.
        assert_eq!(segs.len(), 3);
        assert_eq!(segs[0].text, "See Fig. 3 for details.");
        assert_eq!(segs[1].text, "The voltage must not exceed 24V.");
    }

    #[test]
    fn ui_string_short_no_split() {
        // Single short UI string — common in software localisation
        let segs = segment("File not found.");
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].text, "File not found.");
    }
}
