use serde::{Deserialize, Serialize};

pub mod numbers;
pub mod tags;

/// A single quality-assurance finding for a source/target segment pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QaIssue {
    /// Machine-readable category: `"missing_tag"` or `"missing_number"`.
    pub kind: String,
    /// Human-readable description suitable for display in the UI.
    pub message: String,
}

/// Run all QA checks on a source/target pair and return every issue found.
///
/// Runs tag-consistency and number-consistency checks.  The list is empty
/// when the segment passes all checks.
pub fn run_checks(source: &str, target: &str) -> Vec<QaIssue> {
    let mut issues = Vec::new();
    issues.extend(tags::check(source, target));
    issues.extend(numbers::check(source, target));
    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Basic combined checks ─────────────────────────────────────────────────

    #[test]
    fn clean_pair_has_no_issues() {
        assert!(run_checks("Hello world.", "Hallo Welt.").is_empty());
    }

    #[test]
    fn missing_tag_detected_by_run_checks() {
        let issues = run_checks(
            r#"Click <x id="1"/> to proceed."#,
            "Klicken Sie, um fortzufahren.",
        );
        assert!(issues.iter().any(|i| i.kind == "missing_tag"));
    }

    #[test]
    fn missing_number_detected_by_run_checks() {
        let issues = run_checks("Submit 3 copies.", "Exemplare einreichen.");
        assert!(issues.iter().any(|i| i.kind == "missing_number"));
    }

    #[test]
    fn combined_tag_and_number_issue_both_reported() {
        let issues = run_checks(
            r#"Item <x id="1"/> costs 5 EUR."#,
            "Artikel kostet EUR.",
        );
        let tag_issues = issues.iter().filter(|i| i.kind == "missing_tag").count();
        let num_issues = issues.iter().filter(|i| i.kind == "missing_number").count();
        assert!(tag_issues >= 1, "should have at least one tag issue");
        assert!(num_issues >= 1, "should have at least one number issue");
    }

    // ── Empty input edge cases ────────────────────────────────────────────────

    #[test]
    fn empty_source_and_target_no_issues() {
        assert!(run_checks("", "").is_empty());
    }

    #[test]
    fn empty_source_no_issues() {
        assert!(run_checks("", "Target has 42 items.").is_empty());
    }

    #[test]
    fn empty_target_with_source_content_may_have_issues() {
        // A non-empty source with a tag and a number against empty target
        // should produce at least one issue.
        let issues = run_checks(r#"<x id="1"/> and 42."#, "");
        assert!(!issues.is_empty());
    }

    // ── Real CAT workflow scenarios ───────────────────────────────────────────

    #[test]
    fn software_localisation_string_clean() {
        // UI string with a variable placeholder — translator preserved it.
        assert!(run_checks(
            r#"You have <x id="1"/> unread messages."#,
            r#"Sie haben <x id="1"/> ungelesene Nachrichten."#
        )
        .is_empty());
    }

    #[test]
    fn legal_contract_clause_clean() {
        assert!(run_checks(
            "The penalty shall not exceed 50,000 EUR.",
            "Die Vertragsstrafe darf 50,000 EUR nicht überschreiten.",
        )
        .is_empty());
    }

    #[test]
    fn legal_contract_number_dropped() {
        let issues = run_checks(
            "The penalty shall not exceed 50,000 EUR.",
            "Die Vertragsstrafe darf EUR nicht überschreiten.",
        );
        assert!(issues.iter().any(|i| i.kind == "missing_number"));
    }

    #[test]
    fn technical_manual_with_inline_code() {
        // <ph> is a XLIFF placeholder tag for code.
        assert!(run_checks(
            r#"Run <ph id="1">make install</ph> to complete setup."#,
            r#"Führen Sie <ph id="1">make install</ph> aus, um die Einrichtung abzuschließen."#
        )
        .is_empty());
    }

    #[test]
    fn technical_manual_code_placeholder_missing() {
        let issues = run_checks(
            r#"Run <ph id="1">make install</ph> to complete setup."#,
            "Führen Sie den Befehl aus, um die Einrichtung abzuschließen.",
        );
        assert!(issues.iter().any(|i| i.kind == "missing_tag"));
    }

    #[test]
    fn financial_document_all_numbers_preserved() {
        assert!(run_checks(
            "Invoice #2024-001: Total 1,250.00 USD, VAT 250.00 USD.",
            "Rechnung #2024-001: Gesamt 1,250.00 USD, MwSt. 250.00 USD.",
        )
        .is_empty());
    }

    #[test]
    fn issue_messages_are_human_readable() {
        let issues = run_checks(r#"Item <x id="1"/> costs 99 EUR."#, "Artikel kostet EUR.");
        for issue in &issues {
            assert!(!issue.message.is_empty());
            assert!(issue.message.len() > 10, "message too short: {}", issue.message);
        }
    }
}
