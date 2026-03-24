use regex::Regex;
use std::sync::OnceLock;

use crate::QaIssue;

static NUMBER_RE: OnceLock<Regex> = OnceLock::new();

fn number_regex() -> &'static Regex {
    NUMBER_RE.get_or_init(|| {
        // Matches integers and decimals with period or comma separators.
        // Requires a word boundary so "10" does not match inside "210".
        Regex::new(r"\b\d[\d.,]*\b").unwrap()
    })
}

fn extract_numbers(text: &str) -> Vec<String> {
    number_regex()
        .find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect()
}

/// Check that every number present in `source` also appears in `target`.
///
/// Numbers are matched as literal strings, so locale-specific formatting
/// (e.g. "1,000" vs "1.000") will be flagged.  This is intentional: numbers
/// in professional translation must be adapted explicitly by the translator.
pub fn check(source: &str, target: &str) -> Vec<QaIssue> {
    extract_numbers(source)
        .into_iter()
        .filter(|n| !target.contains(n.as_str()))
        .map(|n| QaIssue {
            kind: "missing_number".to_string(),
            message: format!("Number `{n}` is present in source but missing from target."),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── No numbers ────────────────────────────────────────────────────────────

    #[test]
    fn no_numbers_no_issues() {
        assert!(check("Sign the document.", "Dokument unterzeichnen.").is_empty());
    }

    #[test]
    fn empty_source_no_issues() {
        assert!(check("", "Target text 42.").is_empty());
    }

    #[test]
    fn empty_target_with_source_numbers_flagged() {
        let issues = check("Submit 3 copies.", "");
        assert_eq!(issues.len(), 1);
    }

    // ── Integer matching ──────────────────────────────────────────────────────

    #[test]
    fn matching_integer_no_issue() {
        assert!(check("Submit 3 copies.", "3 Exemplare einreichen.").is_empty());
    }

    #[test]
    fn missing_integer_detected() {
        let issues = check("Submit 3 copies.", "Exemplare einreichen.");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, "missing_number");
        assert!(issues[0].message.contains('3'));
    }

    #[test]
    fn multiple_integers_all_must_be_present() {
        assert!(check(
            "Items 1 and 2 are mandatory.",
            "Punkte 1 und 2 sind obligatorisch."
        )
        .is_empty());
    }

    #[test]
    fn one_of_two_integers_missing() {
        let issues = check("Items 1 and 2 are mandatory.", "Punkt 1 ist obligatorisch.");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains('2'));
    }

    #[test]
    fn all_integers_missing() {
        let issues = check("Items 1 and 2 are mandatory.", "Punkte sind obligatorisch.");
        assert_eq!(issues.len(), 2);
    }

    // ── Decimal numbers ───────────────────────────────────────────────────────

    #[test]
    fn matching_decimal_no_issue() {
        assert!(check("Price: 9.99 EUR.", "Preis: 9.99 EUR.").is_empty());
    }

    #[test]
    fn missing_decimal_detected() {
        let issues = check("Price: 9.99 EUR.", "Preis: EUR.");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("9.99"));
    }

    #[test]
    fn decimal_with_comma_separator() {
        // European locale uses comma as decimal separator.
        // "9,99" is treated as a number token.
        assert!(check("Preis: 9,99 EUR.", "Preis: 9,99 EUR.").is_empty());
    }

    #[test]
    fn different_decimal_format_flagged() {
        // Source has "9.99" (dot), target has "9,99" (comma) — different tokens.
        // This is intentional: the translator must explicitly adapt the format.
        let issues = check("Price: 9.99 EUR.", "Preis: 9,99 EUR.");
        assert_eq!(issues.len(), 1);
    }

    // ── Large numbers ─────────────────────────────────────────────────────────

    #[test]
    fn thousand_separator_number() {
        assert!(check(
            "The contract value is 1,000,000 EUR.",
            "Der Vertragswert beträgt 1,000,000 EUR."
        )
        .is_empty());
    }

    #[test]
    fn large_integer_missing() {
        let issues = check("Contract value: 500000 EUR.", "Vertragswert: EUR.");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("500000"));
    }

    // ── Edge cases ────────────────────────────────────────────────────────────

    #[test]
    fn year_number_must_be_present() {
        assert!(check(
            "The contract expires in 2025.",
            "Der Vertrag läuft 2025 aus."
        )
        .is_empty());
    }

    #[test]
    fn year_number_missing_detected() {
        let issues = check("Expires in 2025.", "Läuft aus.");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("2025"));
    }

    #[test]
    fn number_embedded_in_word_not_extracted() {
        // "24V" — our regex requires a word boundary before the digit.
        // "24" inside "24V" may or may not match depending on word boundaries.
        // The key check: no false positive if the number is genuinely absent.
        let issues = check("Voltage: 24V DC.", "Spannung: 24V DC.");
        assert!(issues.is_empty());
    }

    #[test]
    fn zero_is_checked() {
        let issues = check("Error code: 0.", "Fehlercode:");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains('0'));
    }

    // ── Real CAT scenarios ────────────────────────────────────────────────────

    #[test]
    fn legal_clause_with_days() {
        assert!(check(
            "The seller must deliver within 30 days.",
            "Der Verkäufer muss innerhalb von 30 Tagen liefern."
        )
        .is_empty());
    }

    #[test]
    fn legal_clause_number_omitted_by_translator() {
        let issues = check(
            "The seller must deliver within 30 days.",
            "Der Verkäufer muss innerhalb von Tagen liefern.",
        );
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, "missing_number");
    }

    #[test]
    fn software_version_number() {
        assert!(check(
            "Requires firmware version 3.2.1.",
            "Erfordert Firmware-Version 3.2.1."
        )
        .is_empty());
    }

    #[test]
    fn financial_amount_with_decimals() {
        assert!(check(
            "Total amount due: 1,250.00 USD.",
            "Fälliger Gesamtbetrag: 1,250.00 USD."
        )
        .is_empty());
    }
}
