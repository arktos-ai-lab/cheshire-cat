use regex::Regex;
use std::collections::HashSet;
use std::sync::OnceLock;

use crate::QaIssue;

static TAG_RE: OnceLock<Regex> = OnceLock::new();

fn tag_regex() -> &'static Regex {
    TAG_RE.get_or_init(|| {
        // Matches XLIFF 1.2 and 2.0 inline elements including self-closing variants.
        Regex::new(
            r#"</?(?:x|g|bx|ex|ph|it|mrk|pc|sc|ec|sm|em)\b[^>]*/?>|</(?:x|g|bx|ex|ph|it|mrk|pc|sc|ec|sm|em)>"#,
        )
        .unwrap()
    })
}

fn extract_tags(text: &str) -> HashSet<String> {
    tag_regex()
        .find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect()
}

/// Check that every inline tag present in `source` also appears in `target`.
///
/// Detects missing or extra tags but does not validate ordering, which keeps
/// the check fast and tolerant of legitimate reordering in the translation.
pub fn check(source: &str, target: &str) -> Vec<QaIssue> {
    let src_tags = extract_tags(source);
    let tgt_tags = extract_tags(target);

    let mut issues: Vec<QaIssue> = src_tags
        .difference(&tgt_tags)
        .map(|tag| QaIssue {
            kind: "missing_tag".to_string(),
            message: format!("Tag `{tag}` is present in source but missing from target."),
        })
        .collect();

    // Sort for deterministic output in tests.
    issues.sort_by(|a, b| a.message.cmp(&b.message));
    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Basic tag matching ────────────────────────────────────────────────────

    #[test]
    fn no_tags_no_issues() {
        assert!(check("Hello world.", "Hallo Welt.").is_empty());
    }

    #[test]
    fn matching_x_tag_no_issue() {
        assert!(check(
            r#"Press <x id="1"/>OK."#,
            r#"Drücken Sie <x id="1"/>OK."#
        )
        .is_empty());
    }

    #[test]
    fn matching_g_tag_pair_no_issue() {
        assert!(check(
            r#"<g id="1">Bold text</g> normal."#,
            r#"<g id="1">Fettschrift</g> normal."#
        )
        .is_empty());
    }

    #[test]
    fn missing_x_tag_detected() {
        let issues = check(r#"Press <x id="1"/>OK."#, "Drücken Sie OK.");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, "missing_tag");
        assert!(issues[0].message.contains(r#"<x id="1"/>"#));
    }

    #[test]
    fn missing_g_open_tag_detected() {
        // Target missing the opening <g> tag
        let issues = check(r#"<g id="1">Bold</g> text."#, "Fettschrift text.");
        assert_eq!(issues.len(), 2); // <g id="1"> and </g> both missing
    }

    #[test]
    fn source_has_3_distinct_tags_all_missing() {
        // <g id="1">, </g>, and <x id="2"/> are three distinct tag strings.
        let issues = check(
            r#"<g id="1">Bold</g> and <x id="2"/>plain."#,
            "Fett und Klartext.",
        );
        assert_eq!(issues.len(), 3);
    }

    // ── XLIFF 2.0 tags ────────────────────────────────────────────────────────

    #[test]
    fn xliff2_pc_tag_matched() {
        assert!(check(
            r#"Click <pc id="1">here</pc> to continue."#,
            r#"Klicken Sie <pc id="1">hier</pc>, um fortzufahren."#
        )
        .is_empty());
    }

    #[test]
    fn xliff2_pc_tag_missing_detected() {
        let issues = check(
            r#"Click <pc id="1">here</pc> to continue."#,
            "Klicken Sie hier, um fortzufahren.",
        );
        assert!(!issues.is_empty());
    }

    #[test]
    fn xliff2_sc_ec_tags() {
        assert!(check(
            r#"Start <sc id="1"/>text<ec startRef="1"/> end."#,
            r#"Anfang <sc id="1"/>Text<ec startRef="1"/> Ende."#
        )
        .is_empty());
    }

    // ── Tag reordering tolerance ──────────────────────────────────────────────

    #[test]
    fn reordered_tags_still_pass() {
        // Target has the same tags but in different positions — this is
        // legitimate in many language pairs (e.g. German SOV word order).
        assert!(check(
            r#"The <g id="1">important</g> document was <x id="2"/>signed."#,
            r#"Das Dokument wurde <x id="2"/>mit dem <g id="1">wichtigen</g> Inhalt unterzeichnet."#
        )
        .is_empty());
    }

    // ── Edge cases ────────────────────────────────────────────────────────────

    #[test]
    fn empty_source_and_target_no_issues() {
        assert!(check("", "").is_empty());
    }

    #[test]
    fn empty_source_no_issues() {
        assert!(check("", "Some target text.").is_empty());
    }

    #[test]
    fn plain_html_not_matched() {
        // <b>, <i>, <p> are HTML but not XLIFF inline elements — not checked.
        assert!(check(
            "<b>Bold</b> text.",
            "Fetter Text.",
        )
        .is_empty());
    }

    // ── Real CAT scenarios ────────────────────────────────────────────────────

    #[test]
    fn software_ui_string_with_variable_placeholder() {
        // Common pattern: <x id="1"/> represents a variable like {count}
        assert!(check(
            r#"You have <x id="1"/> unread messages."#,
            r#"Sie haben <x id="1"/> ungelesene Nachrichten."#
        )
        .is_empty());
    }

    #[test]
    fn software_ui_string_translator_forgot_placeholder() {
        let issues = check(
            r#"You have <x id="1"/> unread messages."#,
            "Sie haben ungelesene Nachrichten.",
        );
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, "missing_tag");
    }
}
